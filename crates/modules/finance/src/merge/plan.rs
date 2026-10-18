//! What merging one Account into another does to each of its Transactions. Pure: the one
//! matcher (`dedupe::pair_with`) decides which rows are the same, as it does for a Statement.

use crate::dedupe::{Evidence, StoredEvidence, pair_with};

/// A stored Transaction as matching sees it.
pub type Row = StoredEvidence;

/// What happens to one Transaction of the Account being merged away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    /// No kept Transaction is the same: it moves to the kept Account as it is.
    Repoint {
        /// The Transaction that moves.
        id: i64,
    },
    /// A kept Transaction is the same: its Printings move onto the kept one.
    Fold {
        /// The Transaction folded away.
        from_id: i64,
        /// The kept Transaction that gains its Printings.
        into_id: i64,
    },
}

/// A move for every row of `from`, in order, given the rows of the Account that stays.
#[must_use]
pub fn plan(keep: &[Row], from: &[Row]) -> Vec<Move> {
    let incoming: Vec<Evidence> = from.iter().map(|row| row.evidence.clone()).collect();
    from.iter()
        .zip(pair_with(&incoming, keep))
        .map(|(row, found)| match found {
            Some(into_id) => Move::Fold {
                from_id: row.id,
                into_id,
            },
            None => Move::Repoint { id: row.id },
        })
        .collect()
}
