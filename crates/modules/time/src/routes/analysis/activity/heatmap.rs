//! `GET /api/time/analysis/activity/heatmap`.

use axum::{
    Json,
    extract::{Query, State},
};
use vitarchy_core::AppState;

use super::load;
use crate::{
    analysis::activity::{Heatmap, heatmap},
    error::{ProblemBody, TimeError},
    routes::analysis::params::PeriodQuery,
};

/// Seconds of Activity by weekday (Monday first) and local hour.
#[utoipa::path(
    get,
    path = "/analysis/activity/heatmap",
    tag = "time",
    params(PeriodQuery),
    responses(
        (status = 200, description = "A 7 by 24 grid of seconds", body = Heatmap),
        (status = 400, description = "Bad zone, date or length", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_heatmap(
    State(state): State<AppState>,
    Query(query): Query<PeriodQuery>,
) -> Result<Json<Heatmap>, TimeError> {
    let period = query.period()?;
    Ok(Json(heatmap(&period, &load(&state.pool, &period).await?)))
}
