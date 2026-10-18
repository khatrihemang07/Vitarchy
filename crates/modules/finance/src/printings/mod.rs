//! Printings: one Statement's Printing of a Transaction, kept whole (ADR 0015). The pure rules
//! that turn a Transaction's Printings into the view the person sees live in [`resolve`].

mod gain;
mod resolve;
mod same;

pub use gain::{Plan, plan_for};
pub use resolve::{Resolved, resolve};
pub use same::identical_to_live;

use chrono::NaiveDate;

use crate::reader::contract::Transaction;

/// The seven fields a Statement prints for a Transaction. Two Printings are the same
/// when these are equal (ADR 0015), so they travel as one value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintedFields {
    /// The date as printed.
    pub date: NaiveDate,
    /// The description as printed.
    pub description: String,
    /// Balance after, when printed.
    pub balance_paise: Option<i64>,
    /// A UPI or NEFT reference.
    pub reference: Option<String>,
    /// An id the issuer printed.
    pub source_id: Option<String>,
    /// Foreign amount as printed.
    pub foreign_amount: Option<String>,
    /// Foreign currency code.
    pub foreign_currency: Option<String>,
}

/// One Statement's Printing of a Transaction, exactly as that Statement printed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Printing {
    /// The stored id; 0 for one not stored yet.
    pub id: i64,
    /// The Statement that printed it.
    pub statement_id: i64,
    /// What it printed.
    pub printed: PrintedFields,
    /// Whether a Read again of its Statement no longer finds it.
    pub gone: bool,
}

impl Printing {
    /// The (not yet stored) Printing `statement_id` makes of a row it printed.
    #[must_use]
    pub fn of(statement_id: i64, tx: &Transaction) -> Self {
        Self {
            id: 0,
            statement_id,
            printed: PrintedFields {
                date: tx.date,
                description: tx.description.clone(),
                balance_paise: tx.balance_paise,
                reference: tx.reference.clone(),
                source_id: tx.source_id.clone(),
                foreign_amount: tx.foreign_amount.clone(),
                foreign_currency: tx.foreign_currency.clone(),
            },
            gone: false,
        }
    }
}

#[cfg(test)]
mod tests;
