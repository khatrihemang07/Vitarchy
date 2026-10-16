//! `DELETE /api/time/routines/{id}`.

use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use vitarchy_core::AppState;

use crate::{
    analysis::routines::RoutineError,
    error::{ProblemBody, TimeError},
    repo::routines,
};

/// Deletes a Routine and its Steps. Time Entries are untouched.
#[utoipa::path(
    delete,
    path = "/routines/{id}",
    tag = "time",
    params(("id" = i64, Path, description = "The Routine's id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "No such Routine (`routine_not_found`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn delete_routine(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, TimeError> {
    if !routines::delete(&state.pool, id).await? {
        return Err(RoutineError::NotFound.into());
    }
    tracing::info!(routine_id = id, "routine deleted");
    Ok(StatusCode::NO_CONTENT)
}
