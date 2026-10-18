//! Merging one Account into another inside the caller's transaction.

use sqlx::PgConnection;

use super::plan::{Move, Row, plan};
use crate::{
    import::refresh,
    printings::identical_to_live,
    repo::{merge_moves, printings, transactions},
};

/// What merging one Account did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Merged {
    /// Transactions moved to the kept Account as they were.
    pub moved: usize,
    /// Transactions folded into a kept one (their Printings moved across).
    pub folded: usize,
    /// Printings that moved onto kept Transactions.
    pub printings: u64,
}

/// Merges Account `from_id` into `keep_id` and deletes the emptied `from_id`.
///
/// # Errors
/// The database failed.
pub async fn merge_into(
    db: &mut PgConnection,
    keep_id: i64,
    from_id: i64,
) -> Result<Merged, sqlx::Error> {
    let keep = rows(&mut *db, keep_id).await?;
    let from = rows(&mut *db, from_id).await?;
    let mut done = Merged::default();
    let (mut repoint, mut touched) = (Vec::new(), Vec::new());
    for step in plan(&keep, &from) {
        match step {
            Move::Repoint { id } => repoint.push(id),
            Move::Fold { from_id, into_id } => {
                let both = printings::for_transactions(&mut *db, &[from_id, into_id]).await?;
                let identical = identical_to_live(
                    both.get(&from_id).map_or(&[][..], Vec::as_slice),
                    both.get(&into_id).map_or(&[][..], Vec::as_slice),
                );
                tracing::debug!(
                    from_id,
                    into_id,
                    skipped_identical = identical.len(),
                    "folding transaction"
                );
                done.printings +=
                    merge_moves::fold_transaction(&mut *db, from_id, into_id, &identical).await?;
                done.folded += 1;
                touched.push(into_id);
            }
        }
    }
    done.moved = repoint.len();
    merge_moves::repoint_transactions(&mut *db, &repoint, keep_id).await?;
    refresh::refresh(&mut *db, &touched).await?;
    merge_moves::absorb_account(&mut *db, from_id, keep_id).await?;
    Ok(done)
}

async fn rows(db: &mut PgConnection, account_id: i64) -> Result<Vec<Row>, sqlx::Error> {
    transactions::stored_keys(&mut *db, account_id).await
}
