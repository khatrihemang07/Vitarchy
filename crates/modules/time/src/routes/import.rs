//! `POST /api/time/activity/import`: bring Activity in from the latest Snapshot.

use axum::{Extension, Json, extract::State};
use chrono::Utc;
use vitarchy_core::AppState;

use crate::{
    activity::{ImportCounts, SnapshotFile, run_import},
    error::{ProblemBody, TimeError},
};

/// Runs an Import now and returns what it did.
#[utoipa::path(
    post,
    path = "/activity/import",
    tag = "time",
    responses(
        (status = 200, description = "Import finished", body = ImportCounts),
        (status = 404, description = "No Snapshot yet (`no_snapshot`)", body = ProblemBody),
        (status = 422, description = "The Snapshot could not be read (`snapshot_unreadable`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn post_import(
    State(state): State<AppState>,
    Extension(snapshot): Extension<SnapshotFile>,
) -> Result<Json<ImportCounts>, TimeError> {
    Ok(Json(run_import(&state.pool, &snapshot, Utc::now()).await?))
}
