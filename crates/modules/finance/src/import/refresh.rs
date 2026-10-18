//! Re-deriving Transactions from their Printings (ADR 0015): after Printings are stored or
//! marked, each touched Transaction's columns are resolved again.

use sqlx::PgConnection;

use crate::{
    printings::resolve,
    repo::{printings, transactions_resolved},
};

/// Resolves each Transaction in `ids` again and stores the result.
///
/// # Errors
/// The database failed.
pub async fn refresh(db: &mut PgConnection, ids: &[i64]) -> Result<(), sqlx::Error> {
    let by_transaction = printings::for_transactions(&mut *db, ids).await?;
    for id in ids {
        if let Some(list) = by_transaction.get(id) {
            transactions_resolved::apply(&mut *db, *id, &resolve(list)).await?;
        }
    }
    Ok(())
}
