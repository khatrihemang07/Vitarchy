//! `GET /api/time/analysis/insights`: plain statements found in the person's own data.

use axum::{
    Json,
    extract::{Query, State},
};
use chrono::Utc;
use serde::Serialize;
use utoipa::ToSchema;
use vitarchy_core::AppState;

use super::{activity, lifelog, params::PeriodQuery};
use crate::{
    analysis::insights::{Insight, Sources, gather, insights, reach_period},
    error::{ProblemBody, TimeError},
    repo::routines,
};

/// The most notable Insights for a Period.
#[derive(Serialize, ToSchema)]
pub struct Insights {
    /// Most notable first.
    pub insights: Vec<Insight>,
}

/// Typical times and durations, weekday differences, changes and outliers, computed from one
/// load of Time Entries, Activity and Routines. Changes always compare the last four weeks ending
/// on the Period's last Day with the four before, however short the Period is.
#[utoipa::path(
    get,
    path = "/analysis/insights",
    tag = "time",
    params(PeriodQuery),
    responses(
        (status = 200, description = "Insights, most notable first", body = Insights),
        (status = 400, description = "Bad zone, date or length", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_insights(
    State(state): State<AppState>,
    Query(query): Query<PeriodQuery>,
) -> Result<Json<Insights>, TimeError> {
    let report = query.period()?;
    let reach = reach_period(&report)?;
    let (entries, activity, saved) = tokio::try_join!(
        lifelog::load(&state.pool, &reach),
        activity::load(&state.pool, &reach),
        routines::list(&state.pool),
    )?;
    let sources = Sources {
        entries: &entries,
        activity: &activity,
        routines: &saved,
        now: Utc::now(),
    };
    let found = insights(&gather(&report, &reach, &sources));
    tracing::debug!(
        insights = found.len(),
        routines = saved.len(),
        days = reach.days().len(),
        "insights found"
    );
    Ok(Json(Insights { insights: found }))
}
