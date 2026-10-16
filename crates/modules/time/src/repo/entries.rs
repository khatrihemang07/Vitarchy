//! `time.time_entries`: the read-only mirror of Toggl Time Entries.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use crate::sync::plan::EntryRow;

/// Inserts or updates one entry; `true` when it was new.
pub async fn upsert(tx: &mut PgConnection, row: &EntryRow) -> Result<bool, sqlx::Error> {
    // `xmax = 0` is true only for a freshly inserted row.
    sqlx::query_scalar(
        "INSERT INTO time.time_entries
           (toggl_id, workspace_id, project_id, task_id, description, tags, billable,
            start, stop, duration_s, toggl_updated_at, raw)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
         ON CONFLICT (toggl_id) DO UPDATE SET
           workspace_id = EXCLUDED.workspace_id, project_id = EXCLUDED.project_id,
           task_id = EXCLUDED.task_id, description = EXCLUDED.description,
           tags = EXCLUDED.tags, billable = EXCLUDED.billable, start = EXCLUDED.start,
           stop = EXCLUDED.stop, duration_s = EXCLUDED.duration_s,
           toggl_updated_at = EXCLUDED.toggl_updated_at, raw = EXCLUDED.raw, synced_at = now()
         RETURNING (xmax = 0)",
    )
    .bind(row.toggl_id)
    .bind(row.workspace_id)
    .bind(row.project_id)
    .bind(row.task_id)
    .bind(&row.description)
    .bind(&row.tags)
    .bind(row.billable)
    .bind(row.start)
    .bind(row.stop)
    .bind(row.duration_s)
    .bind(row.toggl_updated_at)
    .bind(&row.raw)
    .fetch_one(tx)
    .await
}

/// Deletes entries by Toggl id; returns how many existed.
pub async fn delete(tx: &mut PgConnection, ids: &[i64]) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("DELETE FROM time.time_entries WHERE toggl_id = ANY($1)")
        .bind(ids)
        .execute(tx)
        .await?;
    Ok(done.rows_affected())
}

/// How much is mirrored, and over what span.
#[derive(Debug, sqlx::FromRow)]
pub struct Stats {
    /// Number of stored entries.
    pub count: i64,
    /// Earliest start.
    pub oldest: Option<DateTime<Utc>>,
    /// Latest start.
    pub newest: Option<DateTime<Utc>>,
}

/// Summary of the stored entries for the status page.
pub async fn stats(pool: &PgPool) -> Result<Stats, sqlx::Error> {
    sqlx::query_as(
        "SELECT count(*) AS count, min(start) AS oldest, max(start) AS newest FROM time.time_entries",
    )
    .fetch_one(pool)
    .await
}
