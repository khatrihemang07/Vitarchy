//! `/api/time/analysis/activity/...`: where screen time goes and when.

pub mod overview;
pub mod timeline;

use sqlx::PgPool;

use crate::{
    analysis::{Period, activity::ActivityRow},
    error::TimeError,
    repo::activities,
};

/// The Activity that touches the Period, prepared for analysis.
pub(crate) async fn load(pool: &PgPool, period: &Period) -> Result<Vec<ActivityRow>, TimeError> {
    let stored = activities::overlapping(pool, period.start(), period.end()).await?;
    tracing::debug!(
        rows = stored.len(),
        days = period.days().len(),
        tz = period.tz().name(),
        "activity loaded for analysis"
    );
    Ok(stored
        .into_iter()
        .map(|row| ActivityRow::new(&row.app, &row.title, row.start_at, row.end_at))
        .collect())
}
