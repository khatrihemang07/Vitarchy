//! `GET /api/time/routines`.

use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;
use vitarchy_core::AppState;

use crate::{analysis::routines::Routine, error::TimeError, repo::routines};

/// The saved Routines.
#[derive(Serialize, ToSchema)]
pub struct RoutineList {
    /// By name.
    pub routines: Vec<Routine>,
}

/// Every saved Routine with its Steps.
#[utoipa::path(
    get,
    path = "/routines",
    tag = "time",
    responses((status = 200, description = "The saved Routines", body = RoutineList))
)]
#[tracing::instrument(skip_all)]
pub async fn get_routines(State(state): State<AppState>) -> Result<Json<RoutineList>, TimeError> {
    let routines = routines::list(&state.pool).await?;
    tracing::debug!(routines = routines.len(), "routines listed");
    Ok(Json(RoutineList { routines }))
}
