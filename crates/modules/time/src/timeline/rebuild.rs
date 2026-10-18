//! Rebuilding the stored Timeline over a stretch of time, from the Activity there.

use std::time::Instant;

use sqlx::{PgConnection, PgPool};

use super::{TimeRange, build, stitch};
use crate::{
    error::TimeError,
    repo::{activities_read, timeline},
};

/// What a rebuild did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rebuilt {
    /// Activity read.
    pub activities: usize,
    /// Segments stored in their place.
    pub segments: usize,
}

/// Rebuilds the Timeline over `range` in its own transaction; see [`rebuild_within`].
///
/// # Errors
/// A failed database read or write; nothing is changed then.
pub async fn rebuild(pool: &PgPool, range: TimeRange) -> Result<Rebuilt, TimeError> {
    let mut tx = pool.begin().await?;
    let rebuilt = rebuild_within(&mut tx, range).await?;
    tx.commit().await?;
    Ok(rebuilt)
}

/// Rebuilds the Timeline over `range` on `tx`, stretched to take in every Activity that
/// overlaps it (from any Source) so no Segment is cut off. The caller commits, so an Import
/// can rebuild in the same transaction as its upsert. Waits for the rebuild lock first. Logs
/// the outcome with counts and timing only, never an app, title or url.
///
/// # Errors
/// A failed database read or write.
pub async fn rebuild_within(tx: &mut PgConnection, range: TimeRange) -> Result<Rebuilt, TimeError> {
    let timer = Instant::now();
    let outcome = replace_stretch(tx, range).await;
    let duration_ms = u64::try_from(timer.elapsed().as_millis()).unwrap_or(u64::MAX);
    match &outcome {
        Ok((range, rebuilt)) => tracing::info!(
            from = %range.from,
            to = %range.to,
            activities = rebuilt.activities,
            segments = rebuilt.segments,
            duration_ms,
            "timeline rebuilt"
        ),
        Err(error) => tracing::warn!(
            from = %range.from,
            to = %range.to,
            error_code = error.code(),
            duration_ms,
            "timeline rebuild failed"
        ),
    }
    outcome.map(|(_, rebuilt)| rebuilt)
}

/// Rebuilds the whole Timeline.
///
/// # Errors
/// As [`rebuild`].
pub async fn rebuild_all(pool: &PgPool) -> Result<Rebuilt, TimeError> {
    rebuild(pool, TimeRange::everything()).await
}

async fn replace_stretch(
    tx: &mut PgConnection,
    wanted: TimeRange,
) -> Result<(TimeRange, Rebuilt), TimeError> {
    timeline::lock_rebuilds(tx).await?;
    let mut range = wanted;
    // Activity or a stored Segment pulled in by widening may itself reach further, so widen
    // until it settles.
    let (found, stored) = loop {
        let found = activities_read::overlapping(&mut *tx, range).await?;
        let stored = timeline::touching(tx, range).await?;
        let widened = found.iter().fold(stitch::reach(range, &stored), |all, a| {
            all.union(TimeRange {
                from: a.start_at,
                to: a.end_at,
            })
        });
        if widened == range {
            break (found, stored);
        }
        range = widened;
    };
    let (replaced, segments) = stitch::stitch(range, build(&found), &stored);
    timeline::replace_range(tx, replaced, &segments).await?;
    Ok((
        replaced,
        Rebuilt {
            activities: found.len(),
            segments: segments.len(),
        },
    ))
}
