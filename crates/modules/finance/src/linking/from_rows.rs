//! Turning stored rows and incoming records into the facts [`super::link`] works on.

use super::facts::{RecordFacts, TransactionFacts};
use crate::{
    reader::contract::{Direction, PaymentRecord},
    repo::{payment_records::PaymentRecordRow, transactions::LinkCandidate},
};

/// The Transactions as linking sees them; rows with an unknown direction are left out.
pub fn transactions(candidates: &[LinkCandidate]) -> Vec<TransactionFacts> {
    candidates
        .iter()
        .filter_map(|candidate| {
            let row = &candidate.row;
            Some(TransactionFacts {
                id: row.id,
                date: row.date,
                amount_paise: row.amount_paise,
                direction: Direction::from_id(&row.direction)?,
                reference: row.reference.clone(),
                description: row.description.clone(),
                account_issuer: candidate.account_issuer.clone(),
                account_mask: candidate.account_mask.clone(),
            })
        })
        .collect()
}

/// Stored Payment records waiting to be linked.
pub fn stored_records(rows: &[PaymentRecordRow]) -> Vec<RecordFacts> {
    rows.iter()
        .filter_map(|row| {
            Some(RecordFacts {
                id: row.id,
                on: row.occurred_at.date(),
                amount_paise: row.amount_paise,
                direction: Direction::from_id(&row.direction)?,
                utr: row.utr.clone(),
                instrument_issuer: row.instrument_issuer.clone(),
                instrument_digits: row.instrument_digits.clone(),
            })
        })
        .collect()
}

/// Incoming records, identified by their index in the file.
pub fn incoming_records(records: &[PaymentRecord]) -> Vec<RecordFacts> {
    records
        .iter()
        .enumerate()
        .map(|(index, record)| RecordFacts {
            id: i64::try_from(index).unwrap_or(i64::MAX),
            on: record.occurred_at.date(),
            amount_paise: record.amount_paise,
            direction: record.direction,
            utr: record.utr.clone(),
            instrument_issuer: record.instrument_issuer.clone(),
            instrument_digits: record.instrument_digits.clone(),
        })
        .collect()
}
