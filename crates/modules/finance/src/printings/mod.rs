//! Printings: one Statement's Printing of a Transaction, kept whole (ADR 0015). The pure rules
//! that turn a Transaction's Printings into the view the person sees live in [`resolve`].

mod gain;
mod resolve;

pub use gain::gains_detail;
pub use resolve::{Resolved, resolve};

use crate::reader::contract::Transaction;

/// One Statement's Printing of a Transaction, exactly as that Statement printed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Printing {
    /// The Statement that printed it.
    pub statement_id: i64,
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
    /// Whether a Read again of its Statement no longer finds it.
    pub gone: bool,
}

impl Printing {
    /// The Printing `statement_id` makes of a row it printed.
    #[must_use]
    pub fn of(statement_id: i64, tx: &Transaction) -> Self {
        Self {
            statement_id,
            description: tx.description.clone(),
            balance_paise: tx.balance_paise,
            reference: tx.reference.clone(),
            source_id: tx.source_id.clone(),
            foreign_amount: tx.foreign_amount.clone(),
            foreign_currency: tx.foreign_currency.clone(),
            gone: false,
        }
    }
}

#[cfg(test)]
mod tests;
