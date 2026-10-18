//! `finance.runs`: one row per read, confirm or Read again attempt, failed ones included.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

/// A stored run.
#[derive(Debug, sqlx::FromRow)]
pub struct RunRow {
    /// `read`, `confirm` or `read_again`.
    pub kind: String,
    /// When it began.
    pub started_at: DateTime<Utc>,
    /// When it ended.
    pub finished_at: DateTime<Utc>,
    /// `success` or `failed`.
    pub outcome: String,
    /// What it did (counts only); `{}` when it failed.
    pub counts: Value,
    /// Why it failed, for failed runs.
    pub error_code: Option<String>,
    /// The Statement it was about, if one.
    pub statement_id: Option<i64>,
}

/// Records a finished run.
///
/// # Errors
/// The database failed.
pub async fn record(pool: &PgPool, run: &RunRow) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO finance.runs (kind, started_at, finished_at, outcome, counts, error_code, statement_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&run.kind)
    .bind(run.started_at)
    .bind(run.finished_at)
    .bind(&run.outcome)
    .bind(&run.counts)
    .bind(&run.error_code)
    .bind(run.statement_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// The most recent run of a kind, whatever its outcome.
///
/// # Errors
/// The database failed.
pub async fn latest(pool: &PgPool, kind: &str) -> Result<Option<RunRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT kind, started_at, finished_at, outcome, counts, error_code, statement_id
         FROM finance.runs WHERE kind = $1 ORDER BY started_at DESC, id DESC LIMIT 1",
    )
    .bind(kind)
    .fetch_optional(pool)
    .await
}

/// Records a finished run of `kind` from its outcome; a failure to record is logged, not
/// raised, so it never hides the run's own result. Counts only, never file content.
pub async fn record_outcome<T: serde::Serialize>(
    pool: &PgPool,
    kind: &str,
    statement_id: Option<i64>,
    started_at: DateTime<Utc>,
    outcome: Result<&T, &crate::FinanceError>,
) {
    let (label, counts, error_code) = match outcome {
        Ok(counts) => (
            "success",
            serde_json::to_value(counts).unwrap_or_default(),
            None,
        ),
        Err(error) => (
            "failed",
            serde_json::json!({}),
            Some(error.code().to_owned()),
        ),
    };
    let row = RunRow {
        kind: kind.to_owned(),
        started_at,
        finished_at: Utc::now(),
        outcome: label.to_owned(),
        counts,
        error_code,
        statement_id,
    };
    if let Err(error) = record(pool, &row).await {
        tracing::error!(?error, kind, "could not record run");
    }
}
