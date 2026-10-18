//! The writes of a Read again for one Account section (ADR 0013, 0015). Nothing is deleted.

use sqlx::PgConnection;

use super::{
    printing_writes::{Target, add_printings, add_transactions},
    read_again_counts::ReadAgainCounts,
    read_again_diff::SectionDiff,
    refresh::refresh,
};
use crate::{reader::contract::Transaction, repo::printings};

/// Applies one section's diff: new Transactions with their first Printing, Printings for
/// Transactions other Statements stored, and the gone marks, then re-resolves what changed.
///
/// # Errors
/// The database failed.
pub(super) async fn write_section(
    db: &mut PgConnection,
    statement_id: i64,
    section: &SectionDiff,
    incoming: &[Transaction],
    counts: &mut ReadAgainCounts,
) -> Result<(), sqlx::Error> {
    let new_rows: Vec<&Transaction> = section.add.iter().map(|i| &incoming[*i]).collect();
    add_transactions(db, section.account_id, statement_id, &new_rows).await?;
    let mut marked = printings::set_gone(&mut *db, statement_id, &section.unchanged, false).await?;
    marked.extend(
        printings::set_gone(&mut *db, statement_id, &section.gone_transaction_ids, true).await?,
    );
    refresh(db, &marked).await?;
    let targets: Vec<Target> = section
        .new_printings
        .iter()
        .map(|p| Target {
            transaction_id: p.transaction_id,
            row: &incoming[p.index],
        })
        .collect();
    let done = add_printings(db, statement_id, &targets).await?;
    counts.enriched += done.enriched;
    counts.printings_added += done.added;
    counts.added += section.add.len();
    counts.unchanged += section.unchanged.len();
    counts.no_longer_found += section.gone_transaction_ids.len();
    Ok(())
}
