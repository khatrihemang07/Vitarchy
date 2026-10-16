//! `/api/time/analysis/lifelog/...`: how the days divide, and what the person's habits are.

pub mod overview;
pub mod timeline;

use sqlx::PgPool;

use crate::{
    analysis::{Period, lifelog::LifeEntry},
    error::TimeError,
    repo::entries,
};

/// The Time Entries that touch the Period, prepared for analysis.
async fn load(pool: &PgPool, period: &Period) -> Result<Vec<LifeEntry>, TimeError> {
    let stored = entries::overlapping(pool, period.start(), period.end()).await?;
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
