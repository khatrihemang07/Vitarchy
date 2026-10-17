//! `time.runs`: one row per Sync or Import attempt, failed ones included.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

use crate::{activity::Source, error::TimeError};

/// A stored run.
#[derive(Debug, sqlx::FromRow)]
pub struct RunRow {
    /// `sync` or `import`.
    pub kind: String,
    /// When it began.
    pub started_at: DateTime<Utc>,
    /// When it ended.
    pub finished_at: DateTime<Utc>,
    /// `success` or `failed`.
    pub outcome: String,
    /// What it did (inserted, updated, ...); `{}` when it failed.
    pub counts: Value,
    /// Why it failed, for failed runs.
    pub error_code: Option<String>,
    /// For an `import`: the Source's id. `None` for a `sync`.
    pub source: Option<String>,
}

/// Records a finished run.
pub async fn insert(pool: &PgPool, run: &RunRow) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO time.runs (kind, started_at, finished_at, outcome, counts, error_code, source)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&run.kind)
    .bind(run.started_at)
    .bind(run.finished_at)
    .bind(&run.outcome)
    .bind(&run.counts)
    .bind(&run.error_code)
    .bind(&run.source)
    .execute(pool)
    .await?;
    Ok(())
}

/// The most recent run of a kind (and, for imports, of one Source), whatever its outcome.
pub async fn latest(
    pool: &PgPool,
    kind: &str,
    source: Option<Source>,
) -> Result<Option<RunRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT kind, started_at, finished_at, outcome, counts, error_code, source FROM time.runs
         WHERE kind = $1 AND source IS NOT DISTINCT FROM $2
         ORDER BY started_at DESC, id DESC LIMIT 1",
    )
    .bind(kind)
    .bind(source.map(Source::id))
    .fetch_optional(pool)
    .await
}

/// Records a finished run of `kind` (an import names its `source`) from its outcome; a failure to record is logged, not
/// raised, so it never hides the run's own result.
pub async fn record<T: serde::Serialize>(
    pool: &PgPool,
    kind: &str,
    source: Option<Source>,
    started_at: DateTime<Utc>,
    outcome: &Result<T, TimeError>,
) {
    let (outcome_label, counts, error_code) = match outcome {
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
        outcome: outcome_label.to_owned(),
        counts,
        error_code,
        source: source.map(|source| source.id().to_owned()),
    };
    if let Err(error) = insert(pool, &row).await {
        tracing::error!(?error, kind, "could not record run");
    }
}
