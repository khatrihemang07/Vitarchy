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
    analysis::insights::{CHANGE_WINDOW_DAYS, Insight, Sources, gather, insights, reach_period},
    error::{ProblemBody, TimeError},
    repo::routines,
};

/// The most notable Insights for a Period.
#[derive(Serialize, ToSchema)]
pub struct Insights {
    /// Most notable first.
    pub insights: Vec<Insight>,
    /// Days in each window a change compares (the last such Days against the same number before).
    pub change_window_days: i64,
}

/// Typical times and durations, weekday differences, changes and outliers, computed from one
/// load of Time Entries, Activity and Routines. Changes always compare the last `change_window_days`
/// ending on the Period's last finished Day with the same number before, however short the Period
/// is. Only finished Days are measured. Typical times, weekday differences and outliers look at
/// the Period's own Days.
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
    let now = Utc::now();
    let reach = reach_period(&report, now)?;
    let (entries, activity, saved) = tokio::try_join!(
        lifelog::load_for_routines(&state.pool, &reach),
        activity::load(&state.pool, &reach),
        routines::list(&state.pool),
    )?;
    let sources = Sources {
        entries: &entries,
        activity: &activity,
        routines: &saved,
        now,
    };
    let found = insights(&gather(&report, &reach, &sources));
    tracing::debug!(
        insights = found.len(),
        routines = saved.len(),
        days = reach.days().len(),
        "insights found"
    );
    Ok(Json(Insights {
        insights: found,
        change_window_days: i64::try_from(CHANGE_WINDOW_DAYS).unwrap_or(i64::MAX),
    }))
}
