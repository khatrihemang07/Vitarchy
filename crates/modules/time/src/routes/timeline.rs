//! `POST /api/time/timeline/rebuild`: rebuild the whole Timeline from stored Activity.

use axum::{Json, extract::State};
use serde::Serialize;
use utoipa::ToSchema;
use vitarchy_core::AppState;

use crate::{
    error::{ProblemBody, TimeError},
    saturating::saturating_u32,
    timeline::rebuild_all,
};

/// What a rebuild produced.
#[derive(Serialize, ToSchema)]
pub struct RebuildResult {
    /// Activity rows read.
    pub activity_count: u32,
    /// Segments now stored.
    pub segment_count: u32,
}

/// Rebuilds the whole Timeline now. Safe to repeat: the Timeline is derived from Activity.
#[utoipa::path(
    post,
    path = "/timeline/rebuild",
    tag = "time",
    responses(
        (status = 200, description = "Timeline rebuilt", body = RebuildResult),
        (status = 500, description = "Internal error", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn post_rebuild(State(state): State<AppState>) -> Result<Json<RebuildResult>, TimeError> {
    let rebuilt = rebuild_all(&state.pool).await?;
    Ok(Json(RebuildResult {
        activity_count: saturating_u32(rebuilt.activities),
        segment_count: saturating_u32(rebuilt.segments),
    }))
}
