//! Reading the stored Timeline for the Timeline page.

use sqlx::PgPool;

use super::timeline::{Row, segment_from};
use chrono::{DateTime, Utc};

use crate::timeline::{TimeRange, model::Segment};

/// Every stored Segment that overlaps `range` (strictly: touching does not count), in time
/// order. Segments crossing the range's edges are returned whole; callers clip them.
pub async fn in_range(pool: &PgPool, range: TimeRange) -> Result<Vec<Segment>, sqlx::Error> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT start_at, end_at, app, title, url, away, sources FROM time.timeline_segments
         WHERE start_at < $2 AND end_at > $1 ORDER BY start_at",
    )
    .bind(range.from)
    .bind(range.to)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(segment_from).collect()
}

/// Where the earliest stored Segment starts, wherever it lies; `None` with no Timeline yet.
/// The Usual measures its history from here, not from the first Segment inside its range.
pub async fn first_start(pool: &PgPool) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar("SELECT MIN(start_at) FROM time.timeline_segments")
        .fetch_one(pool)
        .await
}
