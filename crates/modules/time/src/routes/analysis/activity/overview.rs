//! `GET /api/time/analysis/activity/overview`: every Period-wide Activity analysis from one load.

use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use vitarchy_core::AppState;

use super::load;
use crate::{
    analysis::activity::{DayBound, Heatmap, Totals, Trend, WeekStart, day_bounds, heatmap, totals, trend},
    error::{ProblemBody, TimeError},
    routes::analysis::params::PeriodQuery,
};

/// A Period plus the weekday weeks start on.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct OverviewQuery {
    /// First local date, `YYYY-MM-DD`.
    #[serde(default)]
    pub from: String,
    /// Last local date, inclusive, `YYYY-MM-DD`.
    #[serde(default)]
    pub to: String,
    /// IANA time zone the Days are measured in.
    #[serde(default)]
    pub tz: String,
    /// `monday` (default) or `sunday`.
    #[serde(default)]
    #[param(inline)]
    pub week_start: WeekStart,
}

/// What the Activity tab draws for a Period, all computed from the same rows.
#[derive(Serialize, ToSchema)]
pub struct Overview {
    /// Screen time by app, Context and title.
    pub totals: Totals,
    /// Seconds by weekday and hour.
    pub heatmap: Heatmap,
    /// First and last Activity of each Day that has any, earliest first.
    pub bounds: Vec<DayBound>,
    /// Weekly seconds for the busiest apps.
    pub trend: Trend,
}

/// Totals, heatmap, Day bounds and weekly trend for a Period.
#[utoipa::path(
    get,
    path = "/analysis/activity/overview",
    tag = "time",
    params(OverviewQuery),
    responses(
        (status = 200, description = "Every Period-wide Activity analysis", body = Overview),
        (status = 400, description = "Bad zone, date, length or week start", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_overview(
    State(state): State<AppState>,
    Query(query): Query<OverviewQuery>,
) -> Result<Json<Overview>, TimeError> {
    let period = PeriodQuery {
        from: query.from,
        to: query.to,
        tz: query.tz,
    }
    .period()?;
    let rows = load(&state.pool, &period).await?;
    Ok(Json(Overview {
        totals: totals(&period, &rows),
        heatmap: heatmap(&period, &rows),
        bounds: day_bounds(&period, &rows),
        trend: trend(&period, &rows, query.week_start),
    }))
}
