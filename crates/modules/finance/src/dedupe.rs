//! Telling new Transactions from ones already stored. The same Transaction appears in
//! overlapping Statements and must be stored once, yet two genuinely identical rows in one
//! file are both real, so rows are paired one-to-one by an evidence ladder (id, balance,
//! reference, description) instead of compared by a stored fingerprint.

mod evidence;
mod pairing;
mod verdict;

pub use evidence::Evidence;
pub use pairing::pair;

use crate::reader::contract::Transaction;

/// The evidence of each incoming row.
fn evidence_of(incoming: &[Transaction]) -> Vec<Evidence> {
    incoming.iter().map(Evidence::from).collect()
}

/// Indexes of the incoming rows not already stored, in order. With one row stored, two
/// identical incoming rows add one; a file repeating stored rows adds nothing.
#[must_use]
pub fn new_rows(incoming: &[Transaction], stored: &[Evidence]) -> Vec<usize> {
    pair(&evidence_of(incoming), stored)
        .iter()
        .enumerate()
        .filter(|(_, found)| found.is_none())
        .map(|(index, _)| index)
        .collect()
}

/// A stored Transaction as the Read again diff sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEvidence {
    /// Its row id.
    pub id: i64,
    /// What identifies it.
    pub evidence: Evidence,
    /// Whether an earlier Read again already marked it no longer found.
    pub lost: bool,
}

/// What Read again would change.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReadAgainDiff {
    /// Indexes into the incoming rows that are new.
    pub add: Vec<usize>,
    /// Stored rows the engine still finds (including ones earlier marked lost, which the
    /// caller should clear).
    pub unchanged: Vec<i64>,
    /// Stored rows the engine no longer finds, not already marked.
    pub no_longer_found: Vec<i64>,
}

/// Compares what the engine reads now with what is stored. Nothing is ever deleted: rows
/// the engine no longer finds are only reported. `stored` is in id order, and of twin
/// stored rows the earlier ids are the ones kept as found.
#[must_use]
pub fn read_again_diff(incoming: &[Transaction], stored: &[StoredEvidence]) -> ReadAgainDiff {
    let evidence: Vec<Evidence> = stored.iter().map(|key| key.evidence.clone()).collect();
    let paired = pair(&evidence_of(incoming), &evidence);
    let mut found = vec![false; stored.len()];
    let mut diff = ReadAgainDiff::default();
    for (index, slot) in paired.iter().enumerate() {
        match slot {
            Some(position) => {
                found[*position] = true;
                diff.unchanged.push(stored[*position].id);
            }
            None => diff.add.push(index),
        }
    }
    diff.no_longer_found = stored
        .iter()
        .zip(&found)
        .filter(|(key, found)| !**found && !key.lost)
        .map(|(key, _)| key.id)
        .collect();
    diff
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod verdict_tests;
