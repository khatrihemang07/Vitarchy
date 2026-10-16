//! `GET /api/time/analysis/activity/totals`.

use axum::{
    Json,
    extract::{Query, State},
};
use vitarchy_core::AppState;

use super::load;
use crate::{
    analysis::activity::{Totals, totals},
    error::{ProblemBody, TimeError},
    routes::analysis::params::PeriodQuery,
};

/// Screen time by app, then Context, then top titles.
#[utoipa::path(
    get,
    path = "/analysis/activity/totals",
    tag = "time",
    params(PeriodQuery),
    responses(
        (status = 200, description = "Totals, longest first", body = Totals),
        (status = 400, description = "Bad zone, date or length", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_totals(
    State(state): State<AppState>,
    Query(query): Query<PeriodQuery>,
) -> Result<Json<Totals>, TimeError> {
    let period = query.period()?;
    Ok(Json(totals(&period, &load(&state.pool, &period).await?)))
}
