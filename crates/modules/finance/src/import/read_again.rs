//! Read again for a confirmed Statement: compare what the engine reads now with what is
//! stored, and apply the changes once the person approves (ADR 0013).

use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;

pub use super::read_again_preview::preview;
use super::{
    link_waiting::link_waiting,
    read_again_diff::{Diff, SectionDiff, compute},
    read_again_preview::fresh,
    stored_result::StoredResult,
};
use crate::{
    error::FinanceError,
    files::FileStore,
    problem::Refusal,
    reader::StatementReader,
    repo::{
        confirm_lock, payment_records, runs,
        statements::{self, StatementRow},
        transactions,
    },
};

/// What applying a Read again did.
#[derive(Debug, Clone, Copy, Default, Serialize, ToSchema)]
pub struct ReadAgainCounts {
    /// Rows added.
    pub added: usize,
    /// Stored rows still found.
    pub unchanged: usize,
    /// Rows marked no longer found.
    pub no_longer_found: usize,
    /// Payment records linked as a result.
    pub linked: usize,
}

/// Which confirmed Statements an apply touches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Apply {
    /// Always write, even when nothing changed (the per-Statement approval).
    Always,
    /// Write only when rows would be added or marked (Read all again).
    OnlyChanges,
}

/// Applies a Read again to a confirmed Statement: adds new rows, clears revived marks,
/// marks vanished rows, and keeps the new result.
///
/// # Errors
/// Another confirm is running, the Statement is missing or unconfirmed, the engine can no
/// longer read the file as before, or the file or database failed. Nothing is written then.
pub async fn apply(
    pool: &PgPool,
    reader: &dyn StatementReader,
    files: &FileStore,
    id: i64,
    request_id: Option<&str>,
) -> Result<ReadAgainCounts, FinanceError> {
    let row = statements::get(pool, id)
        .await?
        .ok_or(Refusal::STATEMENT_NOT_FOUND)?;
    let counts = apply_row(pool, reader, files, &row, request_id, Apply::Always).await?;
    Ok(counts.unwrap_or_default())
}

/// [`apply`] for a loaded Statement, recording the run. `None` when `mode` skipped it.
pub(super) async fn apply_row(
    pool: &PgPool,
    reader: &dyn StatementReader,
    files: &FileStore,
    row: &StatementRow,
    request_id: Option<&str>,
    mode: Apply,
) -> Result<Option<ReadAgainCounts>, FinanceError> {
    let started_at = Utc::now();
    let outcome = apply_inner(pool, reader, files, row, request_id, mode).await;
    if let Ok(Some(counts)) = &outcome {
        runs::record_outcome(pool, "read_again", Some(row.id), started_at, Ok(counts)).await;
        tracing::info!(
            statement_id = row.id,
            added = counts.added,
            unchanged = counts.unchanged,
            no_longer_found = counts.no_longer_found,
            linked = counts.linked,
            "read again applied"
        );
    } else if let Err(error) = &outcome {
        runs::record_outcome::<ReadAgainCounts>(
            pool,
            "read_again",
            Some(row.id),
            started_at,
            Err(error),
        )
        .await;
    }
    outcome
}

async fn apply_inner(
    pool: &PgPool,
    reader: &dyn StatementReader,
    files: &FileStore,
    row: &StatementRow,
    request_id: Option<&str>,
    mode: Apply,
) -> Result<Option<ReadAgainCounts>, FinanceError> {
    let id = row.id;
    if row.status != "confirmed" {
        return Err(Refusal::UNCONFIRMED.into());
    }
    let result = fresh(pool, reader, files, row, request_id).await?;
    let mut tx = pool.begin().await?;
    if !confirm_lock::try_acquire(&mut tx).await? {
        return Err(FinanceError::ConfirmInProgress);
    }
    let diff = compute(&mut tx, id, &result).await?;
    if mode == Apply::OnlyChanges && !diff.has_changes() {
        return Ok(None);
    }
    let mut counts = ReadAgainCounts::default();
    match &diff {
        Diff::Sections(sections) => {
            for (section, read) in sections.iter().zip(&result.statements) {
                write_section(&mut tx, id, section, &read.transactions, &mut counts).await?;
            }
        }
        Diff::Records { add, unchanged } => {
            for index in add {
                if payment_records::insert(&mut *tx, id, &result.payment_records[*index]).await? {
                    counts.added += 1;
                }
            }
            counts.unchanged = *unchanged;
        }
    }
    let stored = StoredResult::from_result(&result);
    statements::replace_result(&mut *tx, id, &stored.row(&row.file_sha256, &row.file_name)).await?;
    counts.linked = link_waiting(&mut tx).await?;
    tx.commit().await?;
    Ok(Some(counts))
}

async fn write_section(
    db: &mut sqlx::PgConnection,
    statement_id: i64,
    section: &SectionDiff,
    incoming: &[crate::reader::contract::Transaction],
    counts: &mut ReadAgainCounts,
) -> Result<(), sqlx::Error> {
    for index in &section.add {
        transactions::insert(
            &mut *db,
            section.account_id,
            statement_id,
            &incoming[*index],
        )
        .await?;
    }
    transactions::clear_no_longer_found(&mut *db, &section.unchanged).await?;
    transactions::mark_no_longer_found(&mut *db, &section.no_longer_found).await?;
    counts.added += section.add.len();
    counts.unchanged += section.unchanged.len();
    counts.no_longer_found += section.no_longer_found.len();
    Ok(())
}
