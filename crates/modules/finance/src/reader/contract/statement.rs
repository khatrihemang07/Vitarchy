//! One Account's Statement within a file, row by row.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::enums::{AccountKind, CheckOutcome, Direction};

/// One Account's Statement within a file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statement {
    /// Which Account the reader thinks this is.
    pub account: ProposedAccount,
    /// The printed period.
    pub period: Period,
    /// The printed totals.
    pub summary: Summary,
    /// In printed order.
    pub transactions: Vec<Transaction>,
    /// How the figures agree.
    #[serde(default)]
    pub checks: Vec<Check>,
}

/// The Account as the reader found it; every field may be unknown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposedAccount {
    /// The issuer's name.
    #[serde(default)]
    pub issuer: Option<String>,
    /// Savings, card or loan.
    #[serde(default)]
    pub kind: Option<AccountKind>,
    /// The number as printed, at least the last 4 digits visible.
    #[serde(default)]
    pub mask: Option<String>,
}

/// The printed period; either end may be unknown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Period {
    /// First day.
    #[serde(default)]
    pub start: Option<NaiveDate>,
    /// Last day.
    #[serde(default)]
    pub end: Option<NaiveDate>,
}

/// Figures the Statement prints; each appears only when printed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    /// Balance before the period (signed).
    #[serde(default)]
    pub opening_paise: Option<i64>,
    /// Balance after the period (signed).
    #[serde(default)]
    pub closing_paise: Option<i64>,
    /// Sum of debits.
    #[serde(default)]
    pub total_debit_paise: Option<i64>,
    /// Sum of credits.
    #[serde(default)]
    pub total_credit_paise: Option<i64>,
    /// Number of debits.
    #[serde(default)]
    pub debit_count: Option<u32>,
    /// Number of credits.
    #[serde(default)]
    pub credit_count: Option<u32>,
}

/// One row, exactly as printed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    /// The printed date.
    pub date: NaiveDate,
    /// The printed text, wrapped lines joined.
    pub description: String,
    /// Always greater than 0.
    pub amount_paise: i64,
    /// Which way the money moved.
    pub direction: Direction,
    /// Balance after, when printed (signed).
    #[serde(default)]
    pub balance_paise: Option<i64>,
    /// Foreign-currency amount as printed.
    #[serde(default)]
    pub foreign_amount: Option<String>,
    /// Foreign currency code.
    #[serde(default)]
    pub foreign_currency: Option<String>,
    /// A UPI RRN/UTR or NEFT/IMPS reference found in the description.
    #[serde(default)]
    pub reference: Option<String>,
    /// An id the issuer printed for this row.
    #[serde(default)]
    pub source_id: Option<String>,
}

/// A test of the Statement against itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Check {
    /// `balance_chain`, `summary_totals` or `row_counts`.
    pub name: String,
    /// Passed, failed or skipped.
    pub outcome: CheckOutcome,
    /// Numbers and enum strings only.
    #[serde(default)]
    pub detail: Value,
}
