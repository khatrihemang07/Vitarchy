//! `GET /api/time/analysis/lifelog/overview`: every Period-wide Life log analysis from one load.

use axum::{
    Json,
    extract::{Query, State},
};
use chrono::Utc;
use serde::Serialize;
use utoipa::ToSchema;
use vitarchy_core::AppState;

use super::load;
use crate::{
    analysis::lifelog::{
        DaySplit, DescriptionStat, LifeLogTotals, day_splits, description_stats, totals,
    },
    error::{ProblemBody, TimeError},
    routes::analysis::params::PeriodQuery,
};

/// What the Life log tab draws for a Period, all computed from the same entries.
#[derive(Serialize, ToSchema)]
pub struct LifeLogOverview {
    /// One division per Day of the Period, earliest first.
    pub split: Vec<DaySplit>,
    /// Logged time by project and by description.
    pub totals: LifeLogTotals,
    /// Habits of descriptions seen on at least five Days.
    pub description_stats: Vec<DescriptionStat>,
}

/// Daily split, totals and description habits for a Period.
#[utoipa::path(
    get,
    path = "/analysis/lifelog/overview",
    tag = "time",
    params(PeriodQuery),
    responses(
        (status = 200, description = "Every Period-wide Life log analysis", body = LifeLogOverview),
        (status = 400, description = "Bad zone, date or length", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_lifelog_overview(
    State(state): State<AppState>,
    Query(query): Query<PeriodQuery>,
) -> Result<Json<LifeLogOverview>, TimeError> {
    let period = query.period()?;
    let entries = load(&state.pool, &period).await?;
    Ok(Json(LifeLogOverview {
        split: day_splits(&period, &entries, Utc::now()),
        totals: totals(&period, &entries),
        description_stats: description_stats(&period, &entries),
    }))
}
