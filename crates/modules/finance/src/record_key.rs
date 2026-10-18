//! The identity of a Payment record that has no id of its own from the app: when, how much,
//! which way and with whom. One place defines it for incoming and stored records alike.

use chrono::NaiveDateTime;

use crate::{reader::contract::PaymentRecord, repo::payment_records::PaymentRecordRow};

fn key_of(
    source_id: Option<&str>,
    occurred_at: NaiveDateTime,
    amount_paise: i64,
    direction: &str,
    description: &str,
) -> String {
    match source_id.map(str::trim).filter(|id| !id.is_empty()) {
        Some(id) => format!("id:{id}"),
        None => format!("{occurred_at}|{amount_paise}|{direction}|{description}"),
    }
}

/// The key of an incoming record.
#[must_use]
pub fn of_incoming(record: &PaymentRecord) -> String {
    key_of(
        record.source_id.as_deref(),
        record.occurred_at,
        record.amount_paise,
        record.direction.id(),
        &record.description,
    )
}

/// The key of a stored record.
#[must_use]
pub fn of_stored(row: &PaymentRecordRow) -> String {
    key_of(
        row.source_id.as_deref(),
        row.occurred_at,
        row.amount_paise,
        &row.direction,
        &row.description,
    )
}
