//! Tests for the Printing rules, one file per concern.

use super::{PrintedFields, Printing};
use crate::reader::contract::{Direction, Transaction};

mod gain;
mod resolve;
mod same;

fn printing(description: &str) -> Printing {
    Printing {
        id: 1,
        statement_id: 1,
        printed: PrintedFields {
            date: chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            description: description.to_owned(),
            balance_paise: None,
            reference: None,
            source_id: None,
            foreign_amount: None,
            foreign_currency: None,
        },
        gone: false,
    }
}

fn gone(mut p: Printing) -> Printing {
    p.gone = true;
    p
}

fn row(description: &str, reference: Option<&str>) -> Transaction {
    Transaction {
        date: chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        description: description.to_owned(),
        amount_paise: 100,
        direction: Direction::Debit,
        balance_paise: None,
        foreign_amount: None,
        foreign_currency: None,
        reference: reference.map(str::to_owned),
        source_id: None,
    }
}
