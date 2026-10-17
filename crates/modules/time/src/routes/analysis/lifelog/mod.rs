//! `/api/time/analysis/lifelog/...`: how the days divide, and what the person's habits are.

pub mod overview;
pub mod timeline;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::{
    analysis::{Period, lifelog::LifeEntry, routines::load_window},
    error::TimeError,
    repo::entries,
};

/// The Time Entries that touch the Period, prepared for analysis.
pub(crate) async fn load(pool: &PgPool, period: &Period) -> Result<Vec<LifeEntry>, TimeError> {
    load_between(pool, period, period.start(), period.end()).await
}

/// The Time Entries Routines need for the Period: those that touch it, plus the Days either side,
/// so a Routine that runs past midnight is matched the same at the Period's edges.
pub(crate) async fn load_for_routines(
    pool: &PgPool,
    period: &Period,
) -> Result<Vec<LifeEntry>, TimeError> {
    let (from, to) = load_window(period);
    load_between(pool, period, from, to).await
}

async fn load_between(
    pool: &PgPool,
    period: &Period,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<LifeEntry>, TimeError> {
    let stored = entries::overlapping(pool, from, to).await?;
    tracing::debug!(
        entries = stored.len(),
        days = period.days().len(),
        tz = period.tz().name(),
        "time entries loaded for analysis"
    );
    Ok(stored
        .into_iter()
        .map(|row| {
            LifeEntry::new(
                row.project.as_deref(),
                row.color.as_deref(),
                &row.description,
                row.start_at,
                row.end_at,
            )
        })
        .collect())
}
