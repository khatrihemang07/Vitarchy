//! What a Read again of one Statement would change, decided from evidence alone.

use std::{collections::HashMap, hash::BuildHasher};

use super::{StoredEvidence, pair};
use crate::reader::contract::Transaction;

/// An incoming row that is a Transaction another Statement stored: it gains a Printing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrintingToAdd {
    /// The row's index among the incoming rows.
    pub index: usize,
    /// The stored Transaction it is the same as.
    pub transaction_id: i64,
}

/// What Read again would change.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReadAgainDiff {
    /// Indexes into the incoming rows that are new Transactions.
    pub add: Vec<usize>,
    /// Stored rows this Statement printed and the engine still finds (including ones whose
    /// Printing was marked gone, which the caller should clear).
    pub unchanged: Vec<i64>,
    /// Incoming rows that are stored Transactions this Statement never printed.
    pub new_printings: Vec<PrintingToAdd>,
    /// Transactions whose Printing from this Statement is still live but that the engine
    /// no longer finds: their Printings get marked gone.
    pub gone_transaction_ids: Vec<i64>,
}

/// Compares what the engine reads now with what is stored. `stored` is every row of the
/// Account in id order; `here` holds the rows this Statement printed (id to whether that
/// Printing is gone). Nothing is ever deleted. Rows this Statement printed pair first, so a
/// twin it printed is never swapped for one it did not.
#[must_use]
pub fn read_again_diff<S: BuildHasher>(
    incoming: &[Transaction],
    stored: &[StoredEvidence],
    here: &HashMap<i64, bool, S>,
) -> ReadAgainDiff {
    let mut ordered = stored.to_vec();
    ordered.sort_by_key(|row| !here.contains_key(&row.id));
    let paired = pair(incoming, &ordered);
    let mut diff = ReadAgainDiff::default();
    let mut found = Vec::new();
    for (index, slot) in paired.iter().enumerate() {
        match slot {
            Some(id) if here.contains_key(id) => {
                found.push(*id);
                diff.unchanged.push(*id);
            }
            Some(id) => diff.new_printings.push(PrintingToAdd {
                index,
                transaction_id: *id,
            }),
            None => diff.add.push(index),
        }
    }
    diff.gone_transaction_ids = stored
        .iter()
        .filter(|row| here.get(&row.id) == Some(&false) && !found.contains(&row.id))
        .map(|row| row.id)
        .collect();
    diff
}
