//! Read again for a confirmed Statement: compare what the engine reads now with what is
//! stored, and apply the changes once the person approves (ADR 0013).

use chrono::Utc;
use sqlx::PgPool;

pub use super::read_again_counts::{Apply, ReadAgainCounts};
pub use super::read_again_preview::preview;
use super::{
    read_again_diff::compute, read_again_preview::fresh, read_again_write::write_section,
    stored_result::StoredResult,
};
use crate::{
    error::FinanceError,
    files::FileStore,
    linking::{self, Cause, Trigger},
    problem::Refusal,
    reader::StatementReader,
    repo::{
        confirm_lock, runs, statement_accounts,
        statements::{self, StatementRow},
    },
};

/// Applies a Read again to a confirmed Statement: adds new rows and Printings, clears
/// revived marks, marks vanished Printings, and keeps the new result.
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
    if counts.is_some() {
        linking::after_statement(pool, Cause::in_request(Trigger::ReadAgain, request_id), id).await;
    }
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
            printings_added = counts.printings_added,
            enriched = counts.enriched,
            no_longer_found = counts.no_longer_found,
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
    for slot in &diff.rebuilt {
        statement_accounts::insert(&mut *tx, id, slot).await?;
    }
    if mode == Apply::OnlyChanges && !diff.has_changes() {
        tx.commit().await?;
        return Ok(None);
    }
    let mut counts = ReadAgainCounts::default();
    for (section, read) in diff.sections.iter().zip(&result.statements) {
        write_section(&mut tx, id, section, &read.transactions, &mut counts).await?;
    }
    let stored = StoredResult::from_result(&result);
    statements::replace_result(&mut *tx, id, &stored.row(&row.file_sha256, &row.file_name)).await?;
    tx.commit().await?;
    Ok(Some(counts))
}
