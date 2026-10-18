//! `time.timeline_segments`: the stored Timeline. Rows are derived, so they are replaced
//! wholesale over a range rather than edited.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use crate::{
    activity::Source,
    timeline::{
        TimeRange,
        model::{Away, Segment},
    },
};

/// Advisory-lock key that serialises Timeline rebuilds (an arbitrary constant, "TMLN" in
/// ASCII). Two rebuilds over the same stretch would otherwise both delete and then insert the
/// same Segments and collide on the primary key.
const REBUILD_LOCK_KEY: i64 = 0x544D_4C4E;

/// Waits for the Timeline rebuild lock and holds it until `tx` ends. Taking it again in the
/// same transaction is a no-op, so an Import can take it before its upsert.
pub async fn lock_rebuilds(tx: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(REBUILD_LOCK_KEY)
        .execute(&mut *tx)
        .await?;
    Ok(())
}

/// Replaces every stored Segment that overlaps `range` with `segments`, on `tx`'s connection so
/// the swap is atomic when the caller commits.
pub async fn replace_range(
    tx: &mut PgConnection,
    range: TimeRange,
    segments: &[Segment],
) -> Result<(), sqlx::Error> {
    delete_range(tx, range).await?;
    insert_segments(tx, segments).await
}

/// Deletes every stored Segment that overlaps `range`.
async fn delete_range(tx: &mut PgConnection, range: TimeRange) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM time.timeline_segments WHERE start_at < $2 AND end_at > $1")
        .bind(range.from)
        .bind(range.to)
        .execute(&mut *tx)
        .await?;
    Ok(())
}

/// Stores `segments` in one statement.
async fn insert_segments(tx: &mut PgConnection, segments: &[Segment]) -> Result<(), sqlx::Error> {
    if segments.is_empty() {
        return Ok(());
    }
    let starts: Vec<DateTime<Utc>> = segments.iter().map(|s| s.start_at).collect();
    let ends: Vec<DateTime<Utc>> = segments.iter().map(|s| s.end_at).collect();
    let apps: Vec<&str> = segments.iter().map(|s| s.app.as_str()).collect();
    let titles: Vec<&str> = segments.iter().map(|s| s.title.as_str()).collect();
    let urls: Vec<Option<&str>> = segments.iter().map(|s| s.url.as_deref()).collect();
    let aways: Vec<&str> = segments.iter().map(|s| s.away.as_str()).collect();
    // Source ids never contain a comma, so a joined list round-trips through string_to_array.
    let sources: Vec<String> = segments
        .iter()
        .map(|s| {
            s.sources
                .iter()
                .map(|x| x.id())
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect();
    sqlx::query(
        "INSERT INTO time.timeline_segments (start_at, end_at, app, title, url, away, sources)
         SELECT i.start_at, i.end_at, i.app, i.title, i.url, i.away, string_to_array(i.sources, ',')
         FROM UNNEST($1::timestamptz[], $2::timestamptz[], $3::text[], $4::text[], $5::text[],
                     $6::text[], $7::text[])
           AS i(start_at, end_at, app, title, url, away, sources)",
    )
    .bind(&starts)
    .bind(&ends)
    .bind(&apps)
    .bind(&titles)
    .bind(&urls)
    .bind(&aways)
    .bind(&sources)
    .execute(&mut *tx)
    .await?;
    Ok(())
}

/// A stored Segment as the database returns it.
type Row = (
    DateTime<Utc>,
    DateTime<Utc>,
    String,
    String,
    Option<String>,
    String,
    Vec<String>,
);

/// A stored row as a Segment; an Away word this build does not know is a decode error, not
/// silently Unknown.
fn segment_from(row: Row) -> Result<Segment, sqlx::Error> {
    let (start_at, end_at, app, title, url, away, sources) = row;
    let away = Away::parse(&away)
        .ok_or_else(|| sqlx::Error::Decode(format!("unknown Away value {away:?}").into()))?;
    Ok(Segment {
        start_at,
        end_at,
        app,
        title,
        url,
        away,
        sources: Source::all()
            .filter(|s| sources.iter().any(|id| id == s.id()))
            .collect(),
    })
}

/// Every stored Segment that overlaps `range` or merely touches it, in time order. The touching
/// ones are the neighbours a rebuild must re-join with.
pub async fn touching(
    tx: &mut PgConnection,
    range: TimeRange,
) -> Result<Vec<Segment>, sqlx::Error> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT start_at, end_at, app, title, url, away, sources FROM time.timeline_segments
         WHERE start_at <= $2 AND end_at >= $1 ORDER BY start_at",
    )
    .bind(range.from)
    .bind(range.to)
    .fetch_all(&mut *tx)
    .await?;
    rows.into_iter().map(segment_from).collect()
}

/// How much Timeline is stored, and how much of it is Away or of unknown Away.
#[derive(Debug, sqlx::FromRow)]
pub struct Stats {
    /// Segments stored.
    pub count: i64,
    /// Earliest start.
    pub oldest: Option<DateTime<Utc>>,
    /// Latest end.
    pub newest: Option<DateTime<Utc>>,
    /// Seconds covered by Segments, gaps excluded.
    pub covered_secs: i64,
    /// Seconds of Segments that are Away.
    pub away_secs: i64,
    /// Seconds of Segments whose Away is unknown.
    pub unknown_secs: i64,
}

/// Summary of the stored Timeline for the status page.
pub async fn stats(pool: &PgPool) -> Result<Stats, sqlx::Error> {
    sqlx::query_as(
        "SELECT count(*) AS count, min(start_at) AS oldest, max(end_at) AS newest,
                COALESCE(round(sum(extract(epoch FROM end_at - start_at))), 0)::bigint
                  AS covered_secs,
                COALESCE(round(sum(extract(epoch FROM end_at - start_at))
                               FILTER (WHERE away = 'yes')), 0)::bigint AS away_secs,
                COALESCE(round(sum(extract(epoch FROM end_at - start_at))
                               FILTER (WHERE away = 'unknown')), 0)::bigint AS unknown_secs
         FROM time.timeline_segments",
    )
    .fetch_one(pool)
    .await
}
