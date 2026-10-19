//! Read again over every Statement that still has its original stored.

use sqlx::PgPool;

use super::read_again::{Apply, ReadAgainCounts, apply_row};
use crate::{
    error::FinanceError,
    files::FileStore,
    linking::{self, Cause, Trigger},
    reader::StatementReader,
    repo::statements::{self, StatementRow},
};

/// What applying Read all again did, summed over the Statements it changed.
#[derive(Debug, Clone, Copy, Default, serde::Serialize, utoipa::ToSchema)]
pub struct ReadAllAgainCounts {
    /// Statements that changed and were applied.
    pub statements: usize,
    /// Rows added as new Transactions.
    pub added: usize,
    /// Printings added to Transactions other Statements stored.
    pub printings_added: usize,
    /// Of those, Printings that made the Transaction show more detail.
    pub enriched: usize,
    /// Printings marked no longer found.
    pub no_longer_found: usize,
    /// Statements that could not be read again and were left as they were.
    pub failed: usize,
}

/// The Statements (any status) whose original is still stored, oldest upload first.
///
/// # Errors
/// The database or file store failed.
pub async fn with_originals(
    pool: &PgPool,
    files: &FileStore,
) -> Result<Vec<StatementRow>, FinanceError> {
    let mut rows = statements::list(pool).await?;
    rows.reverse();
    let mut kept = Vec::with_capacity(rows.len());
    for row in rows {
        if files.contains(&row.file_sha256).await? {
            kept.push(row);
        }
    }
    Ok(kept)
}

/// Applies every confirmed Statement's Read again that has changes. A Statement that cannot
/// be read again is counted and skipped; a running confirm stops the whole call.
///
/// # Errors
/// Another confirm is running, or the database or file store failed.
pub async fn apply_all(
    pool: &PgPool,
    reader: &dyn StatementReader,
    files: &FileStore,
    request_id: Option<&str>,
) -> Result<ReadAllAgainCounts, FinanceError> {
    let mut total = ReadAllAgainCounts::default();
    for row in with_originals(pool, files)
        .await?
        .iter()
        .filter(|r| r.status == "confirmed")
    {
        match apply_row(pool, reader, files, row, request_id, Apply::OnlyChanges).await {
            Ok(Some(counts)) => add(&mut total, &counts),
            Ok(None) => {}
            Err(FinanceError::ConfirmInProgress) => return Err(FinanceError::ConfirmInProgress),
            Err(error) => {
                tracing::warn!(
                    statement_id = row.id,
                    code = error.code(),
                    "read again failed"
                );
                total.failed += 1;
            }
        }
    }
    tracing::info!(
        statements = total.statements,
        added = total.added,
        printings_added = total.printings_added,
        enriched = total.enriched,
        no_longer_found = total.no_longer_found,
        failed = total.failed,
        "read all again applied"
    );
    if total.statements > 0 {
        linking::after_everything(pool, Cause::in_request(Trigger::ReadAllAgain, request_id)).await;
    }
    Ok(total)
}

fn add(total: &mut ReadAllAgainCounts, counts: &ReadAgainCounts) {
    total.statements += 1;
    total.added += counts.added;
    total.printings_added += counts.printings_added;
    total.enriched += counts.enriched;
    total.no_longer_found += counts.no_longer_found;
}
