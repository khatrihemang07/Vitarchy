//! `time.activities`: Imported Activity. Rows are only ever added or refreshed, never deleted.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::written::Written;
use crate::activity::parse::Activity;

/// Upserts `activities` in one statement (one transaction). Identical rows are left untouched.
pub async fn upsert_all(pool: &PgPool, activities: &[Activity]) -> Result<Written, sqlx::Error> {
    // Columns bound as parallel arrays; ids and raw travel as text and are cast in SQL.
    let ids: Vec<&str> = activities.iter().map(|a| a.toggl_id.as_str()).collect();
    let clients: Vec<Option<&str>> = activities.iter().map(|a| a.client_id.as_deref()).collect();
    let apps: Vec<&str> = activities.iter().map(|a| a.app.as_str()).collect();
    let titles: Vec<&str> = activities.iter().map(|a| a.title.as_str()).collect();
    let starts: Vec<DateTime<Utc>> = activities.iter().map(|a| a.start_at).collect();
    let ends: Vec<DateTime<Utc>> = activities.iter().map(|a| a.end_at).collect();
    let idles: Vec<bool> = activities.iter().map(|a| a.idle).collect();
    let raws: Vec<String> = activities.iter().map(|a| a.raw.to_string()).collect();
    let counts: (i64, i64) = sqlx::query_as(
        "WITH written AS (
           INSERT INTO time.activities AS a
             (toggl_id, client_id, app, title, start_at, end_at, idle, raw)
           SELECT i.id::uuid, i.client::uuid, i.app, i.title, i.start_at, i.end_at, i.idle, i.raw::jsonb
           FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::timestamptz[],
                       $6::timestamptz[], $7::bool[], $8::text[])
             AS i(id, client, app, title, start_at, end_at, idle, raw)
           ON CONFLICT (toggl_id) DO UPDATE SET
             client_id = EXCLUDED.client_id, app = EXCLUDED.app, title = EXCLUDED.title,
             start_at = EXCLUDED.start_at, end_at = EXCLUDED.end_at, idle = EXCLUDED.idle,
             raw = EXCLUDED.raw, imported_at = now()
           WHERE (a.client_id, a.app, a.title, a.start_at, a.end_at, a.idle, a.raw)
             IS DISTINCT FROM
             (EXCLUDED.client_id, EXCLUDED.app, EXCLUDED.title, EXCLUDED.start_at,
              EXCLUDED.end_at, EXCLUDED.idle, EXCLUDED.raw)
           RETURNING (xmax = 0) AS inserted)
         SELECT count(*) FILTER (WHERE inserted), count(*) FILTER (WHERE NOT inserted) FROM written",
    )
    .bind(&ids)
    .bind(&clients)
    .bind(&apps)
    .bind(&titles)
    .bind(&starts)
    .bind(&ends)
    .bind(&idles)
    .bind(&raws)
    .fetch_one(pool)
    .await?;
    Ok(Written::from_counts(counts))
}

/// How much Activity is stored, and over what span.
#[derive(Debug, sqlx::FromRow)]
pub struct Stats {
    /// Rows stored.
    pub count: i64,
    /// Earliest start.
    pub oldest: Option<DateTime<Utc>>,
    /// Latest end.
    pub newest: Option<DateTime<Utc>>,
    /// Distinct apps.
    pub app_count: i64,
}

/// Summary of stored Activity for the status page.
pub async fn stats(pool: &PgPool) -> Result<Stats, sqlx::Error> {
    sqlx::query_as(
        "SELECT count(*) AS count, min(start_at) AS oldest, max(end_at) AS newest,
                count(DISTINCT app) AS app_count FROM time.activities",
    )
    .fetch_one(pool)
    .await
}

/// A stored row as analysis needs it.
#[derive(Debug, sqlx::FromRow)]
pub struct Stored {
    /// App name as stored.
    pub app: String,
    /// Window title as stored.
    pub title: String,
    /// When it began.
    pub start_at: DateTime<Utc>,
    /// When it ended.
    pub end_at: DateTime<Utc>,
}

/// Rows that overlap `[from, to)`, earliest first. Long rows that began before
/// `from` are included, so a Period's first Day counts them. A row overlapping
/// `from` cannot have started earlier than `from` minus the longest stored row, so
/// `start_at` is bounded on both sides and the `start_at` index is used; the longest
/// row comes from the `activities_span_idx` index in constant time.
pub async fn overlapping(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<Stored>, sqlx::Error> {
    sqlx::query_as(
        "SELECT app, title, start_at, end_at FROM time.activities
         WHERE start_at < $2 AND end_at > $1
           AND start_at > $1 - COALESCE((SELECT max(end_at - start_at) FROM time.activities), interval '0')
         ORDER BY start_at",
    )
    .bind(from)
    .bind(to)
    .fetch_all(pool)
    .await
}
