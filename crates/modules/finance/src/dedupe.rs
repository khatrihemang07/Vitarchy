//! Telling new Transactions from ones already stored. The same Transaction appears in
//! overlapping Statements and must be stored once, yet two genuinely identical rows in one
//! file are both real, so rows are paired one-to-one by an evidence ladder instead of compared
//! by a stored fingerprint. Rows must share an amount and a direction; then, strongest first,
//! a rung present on both rows decides and a disagreement means "not the same":
//!
//! 1. id, only between files of the same kind (bank and payment-app ids do not compare);
//! 2. balance after;
//! 3. reference (a UTR); ids, balances and references allow dates a day apart;
//! 4. description, on the same date;
//! 5. unique amount and date (within a day), only between a payment-app and a bank Printing,
//!    when each is the other's only candidate and no reference or id disagrees.

mod diff;
mod evidence;
mod origin;
mod pairing;
mod verdict;

pub use diff::{PrintingToAdd, ReadAgainDiff, read_again_diff};
pub use evidence::{Evidence, normalise};
pub use origin::{Origin, Origins};

use crate::reader::contract::Transaction;

/// A stored Transaction as matching sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEvidence {
    /// Its row id.
    pub id: i64,
    /// What identifies it: its resolved view.
    pub evidence: Evidence,
    /// What each of its Printings printed, so the row is also the same as a row that matches
    /// one of them (its resolved view may show another Statement's id or text).
    pub printed: Vec<Evidence>,
}

impl StoredEvidence {
    /// A stored row known only by its resolved view.
    #[must_use]
    pub const fn new(id: i64, evidence: Evidence) -> Self {
        Self {
            id,
            evidence,
            printed: Vec::new(),
        }
    }
}

/// For each incoming row of a file of `origin`, the id of the stored Transaction it is the
/// same as. Each stored row pairs at most once, so a file repeating a stored row twice
/// matches it once. This is the one matcher: confirm, preview, Read again and merge all go
/// through it.
#[must_use]
pub fn pair(
    incoming: &[Transaction],
    origin: Origin,
    stored: &[StoredEvidence],
) -> Vec<Option<i64>> {
    let incoming: Vec<Evidence> = incoming
        .iter()
        .map(|tx| Evidence::from(tx).with_origins(Origins::of(origin)))
        .collect();
    pair_with(&incoming, stored)
}

/// [`pair`] for rows whose evidence (origins included) is already built, such as the rows of
/// another stored Account.
#[must_use]
pub fn pair_with(incoming: &[Evidence], stored: &[StoredEvidence]) -> Vec<Option<i64>> {
    let forms: Vec<pairing::Forms<'_>> = stored
        .iter()
        .map(|row| std::iter::once(&row.evidence).chain(&row.printed).collect())
        .collect();
    pairing::pair_forms(incoming, &forms)
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
        .map(|(id, evidence)| StoredEvidence::new(id, evidence.clone()))
        .collect();
    let incoming: Vec<Evidence> = incoming.iter().map(Evidence::from).collect();
    pair_with(&incoming, &numbered)
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
mod unique_tests;
#[cfg(test)]
mod verdict_tests;
