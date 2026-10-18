//! Confirming a Statement: write its new Transactions (or Payment records) and link what can
//! be linked, all in one transaction under the confirm lock.

use std::time::Instant;

use chrono::Utc;
use serde::Serialize;
use sqlx::{PgConnection, PgPool};
use utoipa::ToSchema;

use super::{
    choice::Choice,
    link_waiting::link_waiting,
    resolve::account_for,
    store_rows::{RowCounts, store_rows},
};
use crate::{
    error::FinanceError,
    problem::Refusal,
    reader::contract::{Kind, ReaderResult, Statement},
    repo::{accounts, confirm_lock, payment_records, runs, statement_accounts, statements},
    view::summary::parse_result,
};

/// What a confirm did.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
pub struct ConfirmCounts {
    /// Transactions (or Payment records) stored.
    pub added: usize,
    /// Rows that were already stored: each gained a Printing, not a Transaction.
    pub already_stored: usize,
    /// Of those, rows whose Printing made the Transaction show more detail.
    pub enriched: usize,
    /// Payment records linked to a Transaction by this confirm.
    pub linked: usize,
}

/// Confirms Statement `id` with one choice per Account section.
///
/// # Errors
/// Another confirm is running, the Statement is missing, confirmed or unread, a choice does
/// not fit, or the database failed. Nothing is written then.
pub async fn confirm(
    pool: &PgPool,
    id: i64,
    choices: &[Choice],
) -> Result<ConfirmCounts, FinanceError> {
    let started_at = Utc::now();
    let timer = Instant::now();
    let outcome = apply(pool, id, choices).await;
    runs::record_outcome(pool, "confirm", Some(id), started_at, outcome.as_ref()).await;
    let duration_ms = u64::try_from(timer.elapsed().as_millis()).unwrap_or(u64::MAX);
    match &outcome {
        Ok(counts) => tracing::info!(
            statement_id = id,
            added = counts.added,
            already_stored = counts.already_stored,
            enriched = counts.enriched,
            linked = counts.linked,
            duration_ms,
            "statement confirmed"
        ),
        Err(error) => tracing::warn!(
            statement_id = id,
            error_code = error.code(),
            duration_ms,
            "confirm failed"
        ),
    }
    outcome
}

async fn apply(pool: &PgPool, id: i64, choices: &[Choice]) -> Result<ConfirmCounts, FinanceError> {
    let mut tx = pool.begin().await?;
    if !confirm_lock::try_acquire(&mut tx).await? {
        return Err(FinanceError::ConfirmInProgress);
    }
    let row = statements::get(&mut *tx, id)
        .await?
        .ok_or(Refusal::STATEMENT_NOT_FOUND)?;
    if row.status == "confirmed" {
        return Err(Refusal::CONFIRMED.into());
    }
    let result = parse_result(&row)
        .filter(|r| row.outcome == "read" && r.kind.is_some())
        .ok_or(Refusal::NOT_READABLE)?;
    let rows = if result.kind == Some(Kind::PaymentHistory) {
        write_records(&mut tx, id, &result, choices).await?
    } else {
        write_sections(&mut tx, id, &result.statements, choices).await?
    };
    statements::confirm(&mut *tx, id).await?;
    let linked = link_waiting(&mut tx).await?;
    tx.commit().await?;
    Ok(ConfirmCounts {
        added: rows.added,
        already_stored: rows.already_stored,
        enriched: rows.enriched,
        linked,
    })
}

async fn write_records(
    db: &mut PgConnection,
    id: i64,
    result: &ReaderResult,
    choices: &[Choice],
) -> Result<RowCounts, FinanceError> {
    if !choices.is_empty() {
        return Err(Refusal::SECTIONS_MISMATCH.into());
    }
    let mut added = 0;
    for record in &result.payment_records {
        if payment_records::insert(&mut *db, id, record).await? {
            added += 1;
        }
    }
    Ok(RowCounts {
        added,
        already_stored: result.payment_records.len() - added,
        enriched: 0,
    })
}

async fn write_sections(
    db: &mut PgConnection,
    id: i64,
    sections: &[Statement],
    choices: &[Choice],
) -> Result<RowCounts, FinanceError> {
    if choices.len() != sections.len() {
        return Err(Refusal::SECTIONS_MISMATCH.into());
    }
    let mut total = RowCounts::default();
    for (index, (section, choice)) in sections.iter().zip(choices).enumerate() {
        let known = accounts::identities(&mut *db).await?;
        let account_id = account_for(db, &section.account, choice, &known).await?;
        let position = i32::try_from(index).unwrap_or(i32::MAX);
        statement_accounts::assign(&mut *db, id, position, account_id).await?;
        let counts = store_rows(db, account_id, id, &section.transactions).await?;
        total.added += counts.added;
        total.already_stored += counts.already_stored;
        total.enriched += counts.enriched;
    }
    Ok(total)
}
