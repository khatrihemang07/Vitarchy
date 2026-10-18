//! A Statement as a list row, without its Transactions.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{reader::ReaderResult, repo::statements::StatementRow};

/// One line of the Statements list.
#[derive(Serialize, ToSchema)]
pub struct StatementSummary {
    /// Statement id.
    pub id: i64,
    /// The uploaded file's name.
    pub file_name: String,
    /// `pdf`, `csv`, `xls` or `xlsx`.
    pub format: Option<String>,
    /// `statement` or `payment_history`.
    pub kind: Option<String>,
    /// `read`, `locked`, `unsupported`, `no_table` or `crashed`.
    pub outcome: String,
    /// `unconfirmed` or `confirmed`.
    pub status: String,
    /// When it was uploaded.
    pub uploaded_at: DateTime<Utc>,
    /// When it was confirmed.
    pub confirmed_at: Option<DateTime<Utc>>,
    /// Accounts listed in the file.
    pub account_count: usize,
    /// Transactions (or Payment records) in the file.
    pub row_count: usize,
}

impl StatementSummary {
    /// The list row for a stored Statement.
    #[must_use]
    pub fn of(row: &StatementRow) -> Self {
        let result = parse_result(row);
        let (account_count, row_count) = result.as_ref().map_or((0, 0), |r| {
            let transactions: usize = r.statements.iter().map(|s| s.transactions.len()).sum();
            (r.statements.len(), transactions + r.payment_records.len())
        });
        Self {
            id: row.id,
            file_name: row.file_name.clone(),
            format: row.format.clone(),
            kind: row.kind.clone(),
            outcome: row.outcome.clone(),
            status: row.status.clone(),
            uploaded_at: row.uploaded_at,
            confirmed_at: row.confirmed_at,
            account_count,
            row_count,
        }
    }
}

/// The stored result, if there is one this server can still read.
pub fn parse_result(row: &StatementRow) -> Option<ReaderResult> {
    let value = row.result.clone()?;
    let parsed = serde_json::from_value(value).ok();
    if parsed.is_none() {
        tracing::warn!(statement_id = row.id, "stored result is unreadable");
    }
    parsed
}
