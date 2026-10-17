//! `POST /api/time/activity/{source}/import`: bring one Source's Activity in from its latest Snapshot.

use axum::{
    Extension, Json,
    extract::{Path, State},
};
use chrono::Utc;
use vitarchy_core::{ApiError, AppState};

use crate::{
    activity::{ImportCounts, Snapshots, Source, run_import},
    error::{ProblemBody, TimeError},
};

/// Runs an Import of `source` now and returns what it did.
#[utoipa::path(
    post,
    path = "/activity/{source}/import",
    tag = "time",
    params(("source" = Source, Path, description = "Which Source to Import")),
    responses(
        (status = 200, description = "Import finished", body = ImportCounts),
        (status = 404, description = "Unknown Source, or no Snapshot yet (`no_snapshot`)", body = ProblemBody),
        (status = 422, description = "The Snapshot could not be read (`snapshot_unreadable`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all, fields(source))]
pub async fn post_import(
    State(state): State<AppState>,
    Extension(snapshots): Extension<Snapshots>,
    Path(source): Path<String>,
) -> Result<Json<ImportCounts>, TimeError> {
    // Parsed by hand: the extractor would answer an unknown Source with 400, not 404.
    let source = Source::from_id(&source)
        .ok_or_else(|| ApiError::NotFound("no such Activity Source".to_owned()))?;
    tracing::Span::current().record("source", source.id());
    let file = snapshots.get(source);
    Ok(Json(
        run_import(&state.pool, source, file, Utc::now()).await?,
    ))
}
