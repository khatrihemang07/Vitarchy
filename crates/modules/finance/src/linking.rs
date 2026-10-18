//! Linking Payment records to Transactions (ADR 0014). A record links to at most one
//! Transaction and a Transaction carries at most one record. First by the bank reference
//! (UTR), then by the same amount and direction on a date within a day. Only Transactions
//! of Accounts that agree with the record's instrument are considered; a record that names
//! no instrument links by UTR alone.

mod facts;
pub mod from_rows;
mod matching;

pub use facts::{RecordFacts, TransactionFacts};
use matching::{by_amount_and_date, by_utr, names_instrument};

/// `(record id, transaction id)` pairs. Transactions already carrying a record must not be
/// passed in; each Transaction is used at most once.
#[must_use]
pub fn link(records: &[RecordFacts], transactions: &[TransactionFacts]) -> Vec<(i64, i64)> {
    let mut taken = vec![false; transactions.len()];
    let mut linked: Vec<Option<i64>> = vec![None; records.len()];
    for (slot, record) in records.iter().enumerate() {
        if let Some(index) = by_utr(record, transactions, &taken) {
            taken[index] = true;
            linked[slot] = Some(transactions[index].id);
        }
    }
    for (slot, record) in records.iter().enumerate() {
        if linked[slot].is_some() || !names_instrument(record) {
            continue;
        }
        if let Some(index) = by_amount_and_date(record, transactions, &taken) {
            taken[index] = true;
            linked[slot] = Some(transactions[index].id);
        }
    }
    records
        .iter()
        .zip(linked)
        .filter_map(|(record, tx)| Some((record.id, tx?)))
        .collect()
}

#[cfg(test)]
mod tests;
