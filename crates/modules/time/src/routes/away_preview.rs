//! `GET /api/time/timeline/away-preview`: what each Away threshold would give.

use std::time::Instant;

use axum::{Json, extract::State};
use chrono::{Duration, Utc};
use vitarchy_core::{AppState, Problem};

use super::away_preview_model::AwayPreview;
use crate::{
    error::TimeError,
    repo::activities_read,
    timeline::{
        TimeRange,
        preview::{PREVIEW_DAYS, preview},
    },
};

/// The current Away rules and, for each threshold on offer, the Away and Active time per Day
/// the last `PREVIEW_DAYS` Days of Clockify Activity would have had. Read-only.
#[utoipa::path(
    get,
    path = "/timeline/away-preview",
    tag = "time",
    responses(
        (status = 200, description = "The Away rules and their effect", body = AwayPreview),
        (status = 500, description = "Internal error", body = Problem),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_away_preview(
    State(state): State<AppState>,
) -> Result<Json<AwayPreview>, TimeError> {
    let started = Instant::now();
    let to = Utc::now();
    let range = TimeRange {
        from: to - Duration::days(PREVIEW_DAYS),
        to,
    };
    let activities = activities_read::overlapping(&state.pool, range).await?;
    let result = AwayPreview::new(&preview(&activities));
    tracing::info!(
        activities = activities.len(),
        choices = result.choices.len(),
        duration_ms = started.elapsed().as_millis(),
        "away preview built"
    );
    Ok(Json(result))
}
