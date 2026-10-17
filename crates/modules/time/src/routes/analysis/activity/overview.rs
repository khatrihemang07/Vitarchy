//! `GET /api/time/analysis/activity/overview`: every Period-wide Activity analysis from one load.

use axum::{
    Json,
    extract::{Query, State},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use vitarchy_core::AppState;

use super::load;
use crate::{
    analysis::{
        activity::{
            DayBound, DayTotal, Heatmap, RankedContext, TOP_SUBJECTS, Totals, Trend, WeekStart,
            day_bounds, day_totals, heatmap, top_contexts, totals, trend,
        },
        last_finished_date,
    },
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
    /// The busiest Contexts across apps, longest first.
    pub top_contexts: Vec<RankedContext>,
    /// Screen time of each finished Day that has Activity, earliest first: what an average per
    /// Day is made of.
    pub day_totals: Vec<DayTotal>,
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
    let all = totals(&period, &rows);
    let through = last_finished_date(period.tz(), Utc::now());
    Ok(Json(Overview {
        top_contexts: top_contexts(&all, TOP_SUBJECTS),
        day_totals: day_totals(&period, &rows, through),
        totals: all,
        heatmap: heatmap(&period, &rows),
        bounds: day_bounds(&period, &rows),
        trend: trend(&period, &rows, query.week_start),
    }))
}
