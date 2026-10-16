//! `GET /api/time/analysis/activity/trend`.

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;
use utoipa::IntoParams;
use vitarchy_core::AppState;

use super::load;
use crate::{
    analysis::activity::{Trend, WeekStart, trend},
    error::{ProblemBody, TimeError},
    routes::analysis::params::PeriodQuery,
};

/// A Period plus the weekday weeks start on.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct TrendQuery {
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

/// Weekly totals for the five busiest apps and the rest.
#[utoipa::path(
    get,
    path = "/analysis/activity/trend",
    tag = "time",
    params(TrendQuery),
    responses(
        (status = 200, description = "Weekly seconds per app", body = Trend),
        (status = 400, description = "Bad zone, date or length", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_trend(
    State(state): State<AppState>,
    Query(query): Query<TrendQuery>,
) -> Result<Json<Trend>, TimeError> {
    let period = PeriodQuery {
        from: query.from,
        to: query.to,
        tz: query.tz,
    }
    .period()?;
    Ok(Json(trend(
        &period,
        &load(&state.pool, &period).await?,
        query.week_start,
    )))
}
