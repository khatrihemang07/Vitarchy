//! The Timeline part of the status response.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;

use crate::repo::timeline;

/// The stored Timeline's figures for the status page.
#[derive(Serialize, ToSchema)]
pub struct TimelineStatus {
    /// Segments stored.
    pub segment_count: i64,
    /// Start of the earliest Segment.
    pub oldest: Option<DateTime<Utc>>,
    /// End of the latest Segment.
    pub newest: Option<DateTime<Utc>>,
    /// Seconds the Timeline covers (the sum of its Segments; gaps excluded).
    pub covered_secs: i64,
    /// Seconds of the Timeline that are Away.
    pub away_secs: i64,
    /// Seconds of the Timeline whose Away is unknown.
    pub unknown_secs: i64,
}

/// Reads the stored Timeline's figures.
pub async fn load(pool: &PgPool) -> Result<TimelineStatus, sqlx::Error> {
    let stored = timeline::stats(pool).await?;
    Ok(TimelineStatus {
        segment_count: stored.count,
        oldest: stored.oldest,
        newest: stored.newest,
        covered_secs: stored.covered_secs,
        away_secs: stored.away_secs,
        unknown_secs: stored.unknown_secs,
    })
}
