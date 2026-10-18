//! `time.activities`: Imported Activity from every Source. Rows are only ever added or updated, never deleted.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use super::written::Written;
use crate::activity::{Activity, Source};

/// Upserts `activities` in one statement on `tx`'s connection, keyed by `(source, source_id)`.
/// Identical rows are left untouched. The returned range covers every row written, both where
/// it now sits and where an updated row used to sit, so the Timeline can be rebuilt over both.
pub async fn upsert_all(
    tx: &mut PgConnection,
    activities: &[Activity],
) -> Result<Written, sqlx::Error> {
    // Columns bound as parallel arrays; client ids and raw travel as text and are cast in SQL.
    let sources: Vec<&str> = activities.iter().map(|a| a.source.id()).collect();
    let ids: Vec<&str> = activities.iter().map(|a| a.source_id.as_str()).collect();
    let clients: Vec<Option<&str>> = activities
        .iter()
        .map(|a| a.extras.client_id.as_deref())
        .collect();
    let apps: Vec<&str> = activities.iter().map(|a| a.app.as_str()).collect();
    let titles: Vec<&str> = activities.iter().map(|a| a.title.as_str()).collect();
    let urls: Vec<Option<&str>> = activities.iter().map(|a| a.extras.url.as_deref()).collect();
    let starts: Vec<DateTime<Utc>> = activities.iter().map(|a| a.start_at).collect();
    let ends: Vec<DateTime<Utc>> = activities.iter().map(|a| a.end_at).collect();
    let idles: Vec<Option<bool>> = activities.iter().map(|a| a.extras.idle).collect();
    let idle_secs: Vec<Option<f64>> = activities.iter().map(|a| a.extras.idle_secs).collect();
    let raws: Vec<String> = activities.iter().map(|a| a.raw.to_string()).collect();
    let counts: Counts = sqlx::query_as(
        // `changed_old` and `written` read the same snapshot, so `changed_old` sees the rows as
        // they were before this statement moved or shrank them (Postgres 17 has no RETURNING old).
        "WITH input AS (
           SELECT i.source, i.id, i.client::uuid AS client, i.app, i.title, i.url, i.start_at,
                  i.end_at, i.idle, i.idle_secs, i.raw::jsonb AS raw
           FROM UNNEST($1::text[], $2::text[], $3::text[], $4::text[], $5::text[], $6::text[],
                       $7::timestamptz[], $8::timestamptz[], $9::bool[], $10::float8[], $11::text[])
             AS i(source, id, client, app, title, url, start_at, end_at, idle, idle_secs, raw)),
         changed_old AS (
           SELECT a.start_at, a.end_at FROM time.activities a
           JOIN input i ON a.source = i.source AND a.source_id = i.id
           WHERE (a.client_id, a.app, a.title, a.url, a.start_at, a.end_at, a.idle, a.idle_secs, a.raw)
             IS DISTINCT FROM (i.client, i.app, i.title, i.url, i.start_at, i.end_at, i.idle,
                               i.idle_secs, i.raw)),
         written AS (
           INSERT INTO time.activities AS a
             (source, source_id, client_id, app, title, url, start_at, end_at, idle, idle_secs, raw)
           SELECT source, id, client, app, title, url, start_at, end_at, idle, idle_secs, raw
           FROM input
           ON CONFLICT (source, source_id) DO UPDATE SET
             client_id = EXCLUDED.client_id, app = EXCLUDED.app, title = EXCLUDED.title,
             url = EXCLUDED.url, start_at = EXCLUDED.start_at, end_at = EXCLUDED.end_at,
             idle = EXCLUDED.idle, idle_secs = EXCLUDED.idle_secs, raw = EXCLUDED.raw,
             imported_at = now()
           WHERE (a.client_id, a.app, a.title, a.url, a.start_at, a.end_at, a.idle, a.idle_secs, a.raw)
             IS DISTINCT FROM
             (EXCLUDED.client_id, EXCLUDED.app, EXCLUDED.title, EXCLUDED.url, EXCLUDED.start_at,
              EXCLUDED.end_at, EXCLUDED.idle, EXCLUDED.idle_secs, EXCLUDED.raw)
           RETURNING (xmax = 0) AS inserted, a.start_at, a.end_at)
         SELECT (SELECT count(*) FILTER (WHERE inserted) FROM written),
                (SELECT count(*) FILTER (WHERE NOT inserted) FROM written),
                (SELECT min(start_at) FROM written), (SELECT max(end_at) FROM written),
                (SELECT min(start_at) FROM changed_old), (SELECT max(end_at) FROM changed_old)",
    )
    .bind(&sources)
    .bind(&ids)
    .bind(&clients)
    .bind(&apps)
    .bind(&titles)
    .bind(&urls)
    .bind(&starts)
    .bind(&ends)
    .bind(&idles)
    .bind(&idle_secs)
    .bind(&raws)
    .fetch_one(&mut *tx)
    .await?;
    let (inserted, updated, from, to, old_from, old_to) = counts;
    Ok(Written::from_counts((inserted, updated))
        .covering(from, to)
        .also_covering(old_from, old_to))
}

/// `(inserted, updated, new start, new end, old start, old end)` as the upsert returns them.
type Counts = (
    i64,
    i64,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
);

/// How much Activity is stored, and over what range.
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

/// Summary of one Source's stored Activity for the status page.
pub async fn stats(pool: &PgPool, source: Source) -> Result<Stats, sqlx::Error> {
    sqlx::query_as(
        "SELECT count(*) AS count, min(start_at) AS oldest, max(end_at) AS newest,
                count(DISTINCT app) AS app_count FROM time.activities WHERE source = $1",
    )
    .bind(source.id())
    .fetch_one(pool)
    .await
}
