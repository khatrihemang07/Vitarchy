//! `POST /api/time/sync`: bring the Time Entries from Toggl.

use axum::{Extension, Json, extract::State};
use chrono::Utc;
use vitarchy_core::{ApiError, AppState};

use crate::{
    error::{ProblemBody, TimeError},
    repo::connection,
    sync::{SyncCounts, run_sync},
    toggl::api::TogglFactory,
};

/// Runs a Sync now and returns what it did.
#[utoipa::path(
    post,
    path = "/sync",
    tag = "time",
    responses(
        (status = 200, description = "Sync finished", body = SyncCounts),
        (status = 409, description = "Toggl is not connected (no code), another Sync is running (`sync_in_progress`) or the connection changed meanwhile (`connection_changed`)", body = ProblemBody),
        (status = 401, description = "Toggl rejected the stored token", body = ProblemBody),
        (status = 429, description = "Toggl request limit reached", body = ProblemBody),
        (status = 502, description = "Toggl unreachable or unreadable", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn post_sync(
    State(state): State<AppState>,
    Extension(factory): Extension<TogglFactory>,
) -> Result<Json<SyncCounts>, TimeError> {
    let connection = connection::get(&state.pool)
        .await?
        .ok_or_else(|| ApiError::Conflict("Toggl is not connected.".to_owned()))?;
    let api = factory(&connection.api_token);
    let counts = run_sync(&state.pool, api.as_ref(), &connection, Utc::now()).await?;
    Ok(Json(counts))
}
