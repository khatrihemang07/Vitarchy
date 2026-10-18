//! The shapes of a Statement preview, as the API returns them.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;

use super::transaction::TransactionView;

/// A Statement as the person reviews it before (or after) confirming.
#[derive(Serialize, ToSchema)]
pub struct StatementPreview {
    /// Statement id.
    pub id: i64,
    /// The uploaded file's name.
    pub file_name: String,
    /// `pdf`, `csv`, `xls` or `xlsx`; absent when unsupported.
    pub format: Option<String>,
    /// `statement` or `payment_history`; absent until read.
    pub kind: Option<String>,
    /// `read`, `locked`, `unsupported`, `no_table` or `crashed`.
    pub outcome: String,
    /// `unconfirmed` or `confirmed`.
    pub status: String,
    /// Set when this upload repeated a file already stored: nothing was read again.
    pub duplicate_file: bool,
    /// Where reading stopped and counts, for a file that was not read.
    pub diagnostics: DiagnosticsView,
    /// The printed period across all Accounts in the file.
    pub period: PeriodView,
    /// Whether the file was encrypted.
    pub encrypted: bool,
    /// The label (never the secret) of the Statement password that opened it.
    pub unlocked_with_label: Option<String>,
    /// When it was uploaded.
    pub uploaded_at: DateTime<Utc>,
    /// One section per Account listed (Statements only).
    pub sections: Vec<SectionPreview>,
    /// For a Payment history: its records and what would link.
    pub payment_history: Option<PaymentHistoryPreview>,
}

/// Why a file was not read, without content.
#[derive(Serialize, ToSchema)]
pub struct DiagnosticsView {
    /// `open`, `read`, `locate`, `segment` or `normalize`.
    pub stage: Option<String>,
    /// A stable `snake_case` reason.
    pub code: Option<String>,
    /// Pages read.
    pub pages: Option<u32>,
    /// Rows found.
    pub rows: Option<u32>,
}

/// A printed period; either end may be unknown.
#[derive(Serialize, ToSchema, Default)]
pub struct PeriodView {
    /// First day.
    pub start: Option<NaiveDate>,
    /// Last day.
    pub end: Option<NaiveDate>,
}

/// An existing Account, by id and name.
#[derive(Serialize, ToSchema)]
pub struct AccountRef {
    /// Account id.
    pub id: i64,
    /// The name the person gave it.
    pub name: String,
}

/// The Account the reader thought a section belongs to.
#[derive(Serialize, ToSchema)]
pub struct ProposedAccountView {
    /// The issuer, if found.
    pub issuer: Option<String>,
    /// `savings`, `credit_card` or `loan`, if found.
    pub kind: Option<String>,
    /// The number as printed.
    pub mask: Option<String>,
}

/// A test of the section against itself.
#[derive(Serialize, ToSchema)]
pub struct CheckView {
    /// `balance_chain`, `summary_totals` or `row_counts`.
    pub name: String,
    /// `passed`, `failed` or `skipped`.
    pub outcome: String,
    /// Numbers and enum strings only.
    #[schema(value_type = Object)]
    pub detail: Value,
}

/// One Account's part of a Statement.
#[derive(Serialize, ToSchema)]
pub struct SectionPreview {
    /// Printed order, from 0.
    pub position: i32,
    /// What the reader found.
    pub proposed: ProposedAccountView,
    /// The Account it belongs to, when exactly one fits (or the person chose one).
    pub matched_account: Option<AccountRef>,
    /// True when the file printed no account number and this Account was proposed because it
    /// is the only one with that issuer and kind.
    pub matched_without_number: bool,
    /// When several Accounts fit, which.
    pub candidate_accounts: Vec<AccountRef>,
    /// The printed period.
    pub period: PeriodView,
    /// Transactions in the section.
    pub transaction_count: usize,
    /// Sum of debits.
    pub total_debit_paise: i64,
    /// Sum of credits.
    pub total_credit_paise: i64,
    /// How the figures agree.
    pub checks: Vec<CheckView>,
    /// Rows not stored yet.
    pub new_count: usize,
    /// Rows already stored.
    pub duplicate_count: usize,
    /// The rows, in printed order.
    pub transactions: Vec<TransactionView>,
}

/// What a Payment history holds.
#[derive(Serialize, ToSchema)]
pub struct PaymentHistoryPreview {
    /// Payment records in the file.
    pub record_count: usize,
    /// How many would link to a stored Transaction right now.
    pub would_link_now: usize,
}
