//! Telling new Transactions from ones already stored. The same Transaction appears in
//! overlapping Statements and must be stored once, yet two genuinely identical rows in one
//! file are both real, so rows are paired one-to-one by an evidence ladder (id, balance,
//! reference, description) instead of compared by a stored fingerprint.

mod diff;
mod evidence;
mod pairing;
mod verdict;

pub use diff::{PrintingToAdd, ReadAgainDiff, read_again_diff};
pub use evidence::{Evidence, normalise};

use crate::reader::contract::Transaction;

/// A stored Transaction as matching sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEvidence {
    /// Its row id.
    pub id: i64,
    /// What identifies it: its resolved view.
    pub evidence: Evidence,
}

/// For each incoming row, the id of the stored Transaction it is the same as. Each stored
/// row pairs at most once, so a file repeating a stored row twice matches it once. This is
/// the one matcher: confirm, preview and Read again all go through it.
#[must_use]
pub fn pair(incoming: &[Transaction], stored: &[StoredEvidence]) -> Vec<Option<i64>> {
    let incoming: Vec<Evidence> = incoming.iter().map(Evidence::from).collect();
    let stored_evidence: Vec<Evidence> = stored.iter().map(|row| row.evidence.clone()).collect();
    pairing::pair_evidence(&incoming, &stored_evidence)
        .into_iter()
        .map(|slot| slot.map(|position| stored[position].id))
        .collect()
}

/// Indexes of the incoming rows not already stored, in order. With one row stored, two
/// identical incoming rows add one; a file repeating stored rows adds nothing.
#[must_use]
pub fn new_rows(incoming: &[Transaction], stored: &[Evidence]) -> Vec<usize> {
    let numbered: Vec<StoredEvidence> = (0..)
        .zip(stored)
        .map(|(id, evidence)| StoredEvidence {
            id,
            evidence: evidence.clone(),
        })
        .collect();
    pair(incoming, &numbered)
        .iter()
        .enumerate()
        .filter(|(_, found)| found.is_none())
        .map(|(index, _)| index)
        .collect()
}

#[cfg(test)]
mod diff_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod verdict_tests;
