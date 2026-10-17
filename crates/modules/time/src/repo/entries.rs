//! `time.time_entries`: the read-only mirror of Toggl Time Entries.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use super::written::Written;
use crate::report::entry::WindowEntry;
use crate::sync::plan::{EntryRow, Reconcile};

/// Inserts or updates `rows` (distinct ids) in one statement. Existing rows are
/// always written again, which refreshes `synced_at`.
pub async fn upsert_all(tx: &mut PgConnection, rows: &[EntryRow]) -> Result<Written, sqlx::Error> {
    if rows.is_empty() {
        return Ok(Written::default());
    }
    // Columns bound as parallel arrays. Tags are ragged, so each row's tags travel
    // as a JSON text and are unpacked in SQL; raw travels as text too.
    let ids: Vec<i64> = rows.iter().map(|r| r.toggl_id).collect();
    let workspaces: Vec<i64> = rows.iter().map(|r| r.workspace_id).collect();
    let projects: Vec<Option<i64>> = rows.iter().map(|r| r.project_id).collect();
    let tasks: Vec<Option<i64>> = rows.iter().map(|r| r.task_id).collect();
    let descriptions: Vec<&str> = rows.iter().map(|r| r.description.as_str()).collect();
    let tags: Vec<String> = rows
        .iter()
        .map(|r| serde_json::json!(r.tags).to_string())
        .collect();
    let billable: Vec<bool> = rows.iter().map(|r| r.billable).collect();
    let starts: Vec<DateTime<Utc>> = rows.iter().map(|r| r.start).collect();
    let stops: Vec<DateTime<Utc>> = rows.iter().map(|r| r.stop).collect();
    let durations: Vec<i64> = rows.iter().map(|r| r.duration_s).collect();
    let updated: Vec<Option<DateTime<Utc>>> = rows.iter().map(|r| r.toggl_updated_at).collect();
    let sources: Vec<String> = rows.iter().map(|r| r.raw.to_string()).collect();
    let counts: (i64, i64) = sqlx::query_as(
        "WITH written AS (
           INSERT INTO time.time_entries
             (toggl_id, workspace_id, project_id, task_id, description, tags, billable,
              start, stop, duration_s, toggl_updated_at, raw)
           SELECT i.id, i.workspace_id, i.project_id, i.task_id, i.description,
                  ARRAY(SELECT jsonb_array_elements_text(i.tags::jsonb)), i.billable,
                  i.start, i.stop, i.duration_s, i.updated_at, i.raw::jsonb
           FROM UNNEST($1::bigint[], $2::bigint[], $3::bigint[], $4::bigint[], $5::text[],
                       $6::text[], $7::bool[], $8::timestamptz[], $9::timestamptz[],
                       $10::bigint[], $11::timestamptz[], $12::text[])
             AS i(id, workspace_id, project_id, task_id, description, tags, billable,
                  start, stop, duration_s, updated_at, raw)
           ON CONFLICT (toggl_id) DO UPDATE SET
             workspace_id = EXCLUDED.workspace_id, project_id = EXCLUDED.project_id,
             task_id = EXCLUDED.task_id, description = EXCLUDED.description,
             tags = EXCLUDED.tags, billable = EXCLUDED.billable, start = EXCLUDED.start,
             stop = EXCLUDED.stop, duration_s = EXCLUDED.duration_s,
             toggl_updated_at = EXCLUDED.toggl_updated_at, raw = EXCLUDED.raw,
             synced_at = now()
           RETURNING (xmax = 0) AS inserted)
         SELECT count(*) FILTER (WHERE inserted), count(*) FILTER (WHERE NOT inserted) FROM written",
    )
    .bind(&ids)
    .bind(&workspaces)
    .bind(&projects)
    .bind(&tasks)
    .bind(&descriptions)
    .bind(&tags)
    .bind(&billable)
    .bind(&starts)
    .bind(&stops)
    .bind(&durations)
    .bind(&updated)
    .bind(&sources)
    .fetch_one(tx)
    .await?;
    Ok(Written::from_counts(counts))
}

/// Deletes entries by Toggl id; returns how many existed.
pub async fn delete(tx: &mut PgConnection, ids: &[i64]) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("DELETE FROM time.time_entries WHERE toggl_id = ANY($1)")
        .bind(ids)
        .execute(tx)
        .await?;
    Ok(done.rows_affected())
}

/// Deletes the workspace's entries that start inside the span but are not in `keep`;
/// returns how many went.
pub async fn delete_unlisted(
    tx: &mut PgConnection,
    workspace_id: i64,
    span: &Reconcile,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        "DELETE FROM time.time_entries
         WHERE workspace_id = $1 AND start >= $2 AND start < $3 AND NOT (toggl_id = ANY($4))",
    )
    .bind(workspace_id)
    .bind(span.from)
    .bind(span.to)
    .bind(&span.keep)
    .execute(tx)
    .await?;
    Ok(done.rows_affected())
}

/// Empties the mirror of Time Entries (Toggl owns them, so they can be fetched again).
pub async fn clear(tx: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM time.time_entries")
        .execute(tx)
        .await?;
    Ok(())
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

/// Entries that overlap `[from, to)`, with their project's name and client, oldest first.
/// Overlap (not "starts inside") so an entry crossing the edge still counts for its part.
pub async fn in_range(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<WindowEntry>, sqlx::Error> {
    sqlx::query_as(
        "SELECT e.toggl_id, e.project_id, p.name AS project_name, p.client_id,
                e.description, e.tags, e.start, e.stop
         FROM time.time_entries e
         LEFT JOIN time.projects p ON p.toggl_id = e.project_id
         WHERE e.start < $2 AND e.stop > $1
         ORDER BY e.start, e.toggl_id",
    )
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}
