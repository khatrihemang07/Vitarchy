//! `POST /api/time/routines` and `PUT /api/time/routines/{id}`.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use vitarchy_core::{ApiJson, AppState, Problem};

use super::RoutineRequest;
use crate::{
    analysis::routines::{Routine, RoutineError},
    error::{ProblemBody, TimeError},
    repo::routines,
};

/// Saves a new Routine: confirms a suggestion or creates one from scratch.
#[utoipa::path(
    post,
    path = "/routines",
    tag = "time",
    request_body = RoutineRequest,
    responses(
        (status = 201, description = "Saved", body = Routine),
        (status = 400, description = "Invalid name or Steps (`routine_*` codes)", body = ProblemBody),
        (status = 409, description = "Another Routine has this name (`routine_name_taken`)", body = ProblemBody),
        (status = 422, description = "The body is not a Routine", body = Problem),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn post_routine(
    State(state): State<AppState>,
    ApiJson(body): ApiJson<RoutineRequest>,
) -> Result<(StatusCode, Json<Routine>), TimeError> {
    let draft = body.draft()?;
    let id = routines::create(&state.pool, &draft).await?;
    tracing::info!(routine_id = id, steps = draft.steps.len(), "routine saved");
    let saved = routines::get(&state.pool, id)
        .await?
        .ok_or(RoutineError::NotFound)?;
    Ok((StatusCode::CREATED, Json(saved)))
}

/// Replaces a Routine's name and Steps.
#[utoipa::path(
    put,
    path = "/routines/{id}",
    tag = "time",
    params(("id" = i64, Path, description = "The Routine's id")),
    request_body = RoutineRequest,
    responses(
        (status = 200, description = "Saved", body = Routine),
        (status = 400, description = "Invalid name or Steps (`routine_*` codes)", body = ProblemBody),
        (status = 404, description = "No such Routine (`routine_not_found`)", body = ProblemBody),
        (status = 409, description = "Another Routine has this name (`routine_name_taken`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn put_routine(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    ApiJson(body): ApiJson<RoutineRequest>,
) -> Result<Json<Routine>, TimeError> {
    let draft = body.draft()?;
    routines::replace(&state.pool, id, &draft).await?;
    tracing::info!(
        routine_id = id,
        steps = draft.steps.len(),
        "routine replaced"
    );
    let saved = routines::get(&state.pool, id)
        .await?
        .ok_or(RoutineError::NotFound)?;
    Ok(Json(saved))
}
