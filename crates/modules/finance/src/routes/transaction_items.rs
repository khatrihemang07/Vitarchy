//! The row shapes of the Transactions table.

use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{reader::contract::Direction, repo::transaction_page::PageRow};

/// The Payment record linked to a Transaction.
#[derive(Serialize, ToSchema)]
pub struct LinkedPaymentRecord {
    /// Payment record id.
    pub id: i64,
    /// Local time of the payment.
    pub occurred_at: NaiveDateTime,
    /// With whom.
    pub description: String,
    /// The bank reference.
    pub utr: Option<String>,
    /// The funding account as printed.
    pub instrument: Option<String>,
}

/// One Transaction row.
#[derive(Serialize, ToSchema)]
pub struct TransactionItem {
    /// Transaction id.
    pub id: i64,
    /// Its Account.
    pub account_id: i64,
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
    /// True when a Read again no longer finds this row.
    pub no_longer_found: bool,
    /// The linked Payment record, if any.
    pub payment_record: Option<LinkedPaymentRecord>,
}

/// A page of Transactions.
#[derive(Serialize, ToSchema)]
pub struct TransactionPage {
    /// Rows matching the filter in all.
    pub total: i64,
    /// This page, newest first.
    pub items: Vec<TransactionItem>,
}

impl From<PageRow> for TransactionItem {
    fn from(r: PageRow) -> Self {
        let payment_record = match (r.record_id, r.record_occurred_at, r.record_description) {
            (Some(id), Some(occurred_at), Some(description)) => Some(LinkedPaymentRecord {
                id,
                occurred_at,
                description,
                utr: r.record_utr,
                instrument: r.record_instrument,
            }),
            _ => None,
        };
        Self {
            id: r.id,
            account_id: r.account_id,
            date: r.date,
            description: r.description,
            amount_paise: r.amount_paise,
            direction: r.direction,
            balance_paise: r.balance_paise,
            reference: r.reference,
            foreign_amount: r.foreign_amount,
            foreign_currency: r.foreign_currency,
            no_longer_found: r.no_longer_found,
            payment_record,
        }
    }
}
