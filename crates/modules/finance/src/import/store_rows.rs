//! Writing one Statement's rows for one Account: a row that is a stored Transaction gains a
//! Printing, any other becomes a new Transaction with its first Printing (ADR 0015).

use sqlx::PgConnection;

use super::printing_writes::{Target, add_printings, add_transactions};
use crate::{
    dedupe::{Origin, pair},
    reader::contract::Transaction,
    repo::transactions,
};

/// What storing a section's rows did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RowCounts {
    /// Rows stored as new Transactions.
    pub added: usize,
    /// Rows that were already stored Transactions.
    pub already_stored: usize,
    /// Of those, rows whose Printing changed what the Transaction shows.
    pub enriched: usize,
}

/// Stores `rows` of Statement `statement_id` (a file of `origin`) under the Account.
///
/// # Errors
/// The database failed.
pub async fn store_rows(
    db: &mut PgConnection,
    account_id: i64,
    statement_id: i64,
    origin: Origin,
    rows: &[Transaction],
) -> Result<RowCounts, sqlx::Error> {
    let stored = transactions::stored_keys(&mut *db, account_id).await?;
    let mut new_rows = Vec::new();
    let mut targets = Vec::new();
    for (row, found) in rows.iter().zip(pair(rows, origin, &stored)) {
        match found {
            Some(transaction_id) => targets.push(Target {
                transaction_id,
                row,
            }),
            None => new_rows.push(row),
        }
    }
    let added = add_transactions(db, account_id, statement_id, &new_rows).await?;
    let done = add_printings(db, statement_id, &targets).await?;
    Ok(RowCounts {
        added,
        already_stored: targets.len(),
        enriched: done.enriched,
    })
}
