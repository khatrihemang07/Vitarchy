//! What linking works on: Payment records and Transactions reduced to the fields it compares.

use chrono::NaiveDate;

use crate::reader::contract::Direction;

/// A Payment record as linking sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordFacts {
    /// Its row id.
    pub id: i64,
    /// The day it happened (local).
    pub on: NaiveDate,
    /// The amount.
    pub amount_paise: i64,
    /// Which way the money moved.
    pub direction: Direction,
    /// The bank reference, if the app noted one.
    pub utr: Option<String>,
    /// The issuer recognised in the funding account, if any.
    pub instrument_issuer: Option<String>,
    /// The trailing visible digits of the funding account, if any.
    pub instrument_digits: Option<String>,
}

/// A Transaction that may take a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionFacts {
    /// Its row id.
    pub id: i64,
    /// Its date.
    pub date: NaiveDate,
    /// The amount.
    pub amount_paise: i64,
    /// Which way the money moved.
    pub direction: Direction,
    /// The reference found in its description.
    pub reference: Option<String>,
    /// Its description as printed.
    pub description: String,
    /// The issuer of the Account it belongs to.
    pub account_issuer: String,
    /// The number of the Account it belongs to, as printed.
    pub account_mask: Option<String>,
}
