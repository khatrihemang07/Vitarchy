//! Builders for unit tests of the pure logic.

use chrono::NaiveDate;

use crate::reader::contract::{Direction, Transaction};

/// A debit of `amount_paise` on 2026-07-02 with the given description.
pub fn tx(description: &str, amount_paise: i64) -> Transaction {
    Transaction {
        date: NaiveDate::from_ymd_opt(2026, 7, 2).expect("valid date"),
        description: description.to_owned(),
        amount_paise,
        direction: Direction::Debit,
        balance_paise: None,
        foreign_amount: None,
        foreign_currency: None,
        reference: None,
        addon_card: None,
        source_id: None,
    }
}
