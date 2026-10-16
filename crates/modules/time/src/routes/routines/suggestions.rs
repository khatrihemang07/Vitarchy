//! `GET /api/time/routines/suggestions`.

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Serialize;
use utoipa::ToSchema;
use vitarchy_core::AppState;

use crate::{
    analysis::routines::{MatchKind, Suggestion, suggest},
    error::{ProblemBody, TimeError},
    repo::routines,
    routes::analysis::{lifelog::load, params::PeriodQuery},
};

/// Recurring sequences the person has not saved as Routines.
#[derive(Serialize, ToSchema)]
pub struct Suggestions {
    /// Best first.
    pub suggestions: Vec<Suggestion>,
}

/// Sequences of descriptions that recur in order within a Day, over at least five Days.
#[utoipa::path(
    get,
    path = "/routines/suggestions",
    tag = "time",
    params(PeriodQuery),
    responses(
        (status = 200, description = "Suggested Routines", body = Suggestions),
        (status = 400, description = "Bad zone, date or length", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_suggestions(
    State(state): State<AppState>,
    Query(query): Query<PeriodQuery>,
) -> Result<Json<Suggestions>, TimeError> {
    let period = query.period()?;
    let entries = load(&state.pool, &period).await?;
    let saved: Vec<Vec<String>> = routines::list(&state.pool)
        .await?
        .into_iter()
        .map(|routine| {
            routine
                .steps
                .into_iter()
                .filter(|step| step.kind == MatchKind::Description)
                .map(|step| step.value)
                .collect()
        })
        .collect();
    let suggestions = suggest(&period, &entries, &saved);
    tracing::debug!(suggestions = suggestions.len(), "routines suggested");
    Ok(Json(Suggestions { suggestions }))
}
