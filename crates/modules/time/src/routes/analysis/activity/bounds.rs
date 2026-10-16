//! `GET /api/time/analysis/activity/bounds`.

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Serialize;
use utoipa::ToSchema;
use vitarchy_core::AppState;

use super::load;
use crate::{
    analysis::activity::{DayBound, day_bounds},
    error::{ProblemBody, TimeError},
    routes::analysis::params::PeriodQuery,
};

/// First and last Activity of each Day that has any.
#[derive(Serialize, ToSchema)]
pub struct Bounds {
    /// Days with Activity, earliest first.
    pub days: Vec<DayBound>,
}

/// When each Day's Activity began and ended, in local minutes.
#[utoipa::path(
    get,
    path = "/analysis/activity/bounds",
    tag = "time",
    params(PeriodQuery),
    responses(
        (status = 200, description = "Per-Day first and last Activity", body = Bounds),
        (status = 400, description = "Bad zone, date or length", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_bounds(
    State(state): State<AppState>,
    Query(query): Query<PeriodQuery>,
) -> Result<Json<Bounds>, TimeError> {
    let period = query.period()?;
    Ok(Json(Bounds {
        days: day_bounds(&period, &load(&state.pool, &period).await?),
    }))
}
