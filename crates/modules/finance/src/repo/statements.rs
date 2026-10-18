//! `finance.statements`: one row per uploaded file.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgExecutor;

pub use super::{
    statement_changes::{confirm, discard, replace_result},
    statement_files::file_in_use,
};

/// A stored Statement.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StatementRow {
    /// Row id.
    pub id: i64,
    /// Hash of the original (the file store's name for it).
    pub file_sha256: String,
    /// The uploaded file's name.
    pub file_name: String,
    /// `pdf`, `csv`, `xls` or `xlsx`; unknown when unsupported.
    pub format: Option<String>,
    /// `statement` or `payment_history`; set once read.
    pub kind: Option<String>,
    /// `unconfirmed` or `confirmed`.
    pub status: String,
    /// `read`, `locked`, `unsupported`, `no_table` or `crashed`.
    pub outcome: String,
    /// The reader's full result.
    pub result: Option<Value>,
    /// When it was uploaded.
    pub uploaded_at: DateTime<Utc>,
    /// When the person confirmed it.
    pub confirmed_at: Option<DateTime<Utc>>,
    /// When the reader last read it.
    pub read_at: Option<DateTime<Utc>>,
    /// Which reader engine produced `result`.
    pub reader_version: Option<String>,
}

/// What a new Statement row starts with.
#[derive(Debug)]
pub struct NewStatement<'a> {
    /// Hash of the stored original.
    pub file_sha256: &'a str,
    /// The uploaded file's name.
    pub file_name: &'a str,
    /// Wire spelling of the format, if known.
    pub format: Option<&'a str>,
    /// Wire spelling of the kind, if read.
    pub kind: Option<&'a str>,
    /// Wire spelling of the outcome (or `crashed`).
    pub outcome: &'a str,
    /// The reader's result as JSON.
    pub result: Option<&'a Value>,
    /// The reader engine's version.
    pub reader_version: Option<&'a str>,
}

pub(super) const COLUMNS: &str = "id, file_sha256, file_name, format, kind, status, outcome, result, uploaded_at, confirmed_at, read_at, reader_version";

/// Adds an unconfirmed Statement and returns its id.
///
/// # Errors
/// The database failed.
pub async fn insert(db: impl PgExecutor<'_>, new: &NewStatement<'_>) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO finance.statements (file_sha256, file_name, format, kind, outcome, result, read_at, reader_version)
         VALUES ($1, $2, $3, $4, $5, $6, now(), $7) RETURNING id",
    )
    .bind(new.file_sha256)
    .bind(new.file_name)
    .bind(new.format)
    .bind(new.kind)
    .bind(new.outcome)
    .bind(new.result)
    .bind(new.reader_version)
    .fetch_one(db)
    .await
}

/// One Statement.
///
/// # Errors
/// The database failed.
pub async fn get(db: impl PgExecutor<'_>, id: i64) -> Result<Option<StatementRow>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM finance.statements WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(db)
    .await
}

/// Every Statement, newest upload first.
///
/// # Errors
/// The database failed.
pub async fn list(db: impl PgExecutor<'_>) -> Result<Vec<StatementRow>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM finance.statements ORDER BY uploaded_at DESC, id DESC"
    )))
    .fetch_all(db)
    .await
}

/// Statements with this status, newest upload first.
///
/// # Errors
/// The database failed.
pub async fn list_with_status(
    db: impl PgExecutor<'_>,
    status: &str,
) -> Result<Vec<StatementRow>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM finance.statements WHERE status = $1 ORDER BY uploaded_at DESC, id DESC"
    )))
    .bind(status)
    .fetch_all(db)
    .await
}
