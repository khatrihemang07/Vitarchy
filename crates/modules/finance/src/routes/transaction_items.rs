//! The row shapes of the Transactions table.

use chrono::{NaiveDate, NaiveTime};
use serde::Serialize;
use utoipa::ToSchema;

use super::transfer_info::TransferInfo;
use crate::{reader::contract::Direction, repo::transaction_page::PageRow, transfers::Parts};

/// One Transaction row.
#[derive(Serialize, ToSchema)]
pub struct TransactionItem {
    /// Transaction id.
    pub id: i64,
    /// Its Account.
    pub account_id: i64,
    /// The printed date.
    pub date: NaiveDate,
    /// The local time, when any Statement printed one (payment apps do).
    pub time: Option<NaiveTime>,
    /// The fullest description any Statement printed.
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
    /// True when a Read again no longer finds this row in any Statement that printed it.
    pub no_longer_found: bool,
    /// How many Statements printed it; above 1, `GET /transactions/{id}/printings` lists them.
    pub printing_count: i64,
    /// The Transfer this row is part of, or null. A Transfer is money moving between two of
    /// the person's own Accounts, so it is neither Money in nor Money out.
    pub transfer: Option<TransferInfo>,
}

/// A page of Transactions.
#[derive(Serialize, ToSchema)]
pub struct TransactionPage {
    /// Rows matching the filter in all.
    pub total: i64,
    /// This page, newest first.
    pub items: Vec<TransactionItem>,
}

impl TransactionItem {
    /// The row, with its Transfer (if any) taken from `transfers`.
    #[must_use]
    pub fn of(r: PageRow, transfers: &[Parts]) -> Self {
        let transfer = r
            .transfer_id
            .and_then(|id| transfers.iter().find(|t| t.id == id))
            .map(|parts| TransferInfo::of(parts, r.account_id));
        Self {
            transfer,
            ..Self::from(r)
        }
    }
}

impl From<PageRow> for TransactionItem {
    fn from(r: PageRow) -> Self {
        Self {
            transfer: None,
            id: r.id,
            account_id: r.account_id,
            date: r.date,
            time: r.time,
            description: r.description,
            amount_paise: r.amount_paise,
            direction: r.direction,
            balance_paise: r.balance_paise,
            reference: r.reference,
            foreign_amount: r.foreign_amount,
            foreign_currency: r.foreign_currency,
            no_longer_found: r.no_longer_found,
            printing_count: r.printing_count,
        }
    }
}
