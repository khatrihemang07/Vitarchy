//! What merging one Account into another does to each of its Transactions. Pure: the one
//! matcher (`dedupe::pair`) decides which rows are the same, as it does for a Statement.

use crate::{
    dedupe::{Evidence, StoredEvidence, pair},
    reader::contract::Transaction,
};

/// A stored Transaction as its resolved view reads.
#[derive(Debug, Clone)]
pub struct Row {
    /// Its row id.
    pub id: i64,
    /// What it shows.
    pub tx: Transaction,
}

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
    let stored: Vec<StoredEvidence> = keep
        .iter()
        .map(|row| StoredEvidence {
            id: row.id,
            evidence: Evidence::from(&row.tx),
        })
        .collect();
    let incoming: Vec<Transaction> = from.iter().map(|row| row.tx.clone()).collect();
    from.iter()
        .zip(pair(&incoming, &stored))
        .map(|(row, found)| match found {
            Some(into_id) => Move::Fold {
                from_id: row.id,
                into_id,
            },
            None => Move::Repoint { id: row.id },
        })
        .collect()
}
