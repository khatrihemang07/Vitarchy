//! One row of a Statement as the preview and the change list show it.

use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;

use crate::reader::contract::{Direction, Transaction};

/// A Transaction as printed.
#[derive(Serialize, ToSchema)]
pub struct TransactionView {
    /// The printed date.
    pub date: NaiveDate,
    /// The printed description.
    pub description: String,
    /// Amount in paise, above 0.
    pub amount_paise: i64,
    /// Which way the money moved.
    pub direction: Direction,
    /// Balance after, when printed.
    pub balance_paise: Option<i64>,
    /// A UPI or NEFT reference.
    pub reference: Option<String>,
    /// Foreign amount as printed.
    pub foreign_amount: Option<String>,
    /// Foreign currency code.
    pub foreign_currency: Option<String>,
    /// The add-on card, on a card bill.
    pub addon_card: Option<String>,
    /// Whether an equal row is already stored.
    pub is_duplicate: bool,
}

impl TransactionView {
    /// The view of `tx`, marked a duplicate or not.
    #[must_use]
    pub fn of(tx: &Transaction, is_duplicate: bool) -> Self {
        Self {
            date: tx.date,
            description: tx.description.clone(),
            amount_paise: tx.amount_paise,
            direction: tx.direction,
            balance_paise: tx.balance_paise,
            reference: tx.reference.clone(),
            foreign_amount: tx.foreign_amount.clone(),
            foreign_currency: tx.foreign_currency.clone(),
            addon_card: tx.addon_card.clone(),
            is_duplicate,
        }
    }
}
