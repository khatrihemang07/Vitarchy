//! `GET /api/time/routines/{id}/occurrences`.

use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Serialize;
use utoipa::ToSchema;
use vitarchy_core::AppState;

use crate::{
    analysis::routines::{
        Occurrence, Routine, RoutineError, RoutineStats, occurrences, routine_stats,
    },
    error::{ProblemBody, TimeError},
    repo::routines,
    routes::analysis::{lifelog::load, params::PeriodQuery},
};

/// How a Routine went over a Period.
#[derive(Serialize, ToSchema)]
pub struct RoutineOccurrences {
    /// The Routine.
    pub routine: Routine,
    /// Each Day it happened, earliest first.
    pub occurrences: Vec<Occurrence>,
    /// The usual times and lengths.
    pub stats: RoutineStats,
}

/// A Routine's Occurrences, complete and incomplete, with statistics.
#[utoipa::path(
    get,
    path = "/routines/{id}/occurrences",
    tag = "time",
    params(("id" = i64, Path, description = "The Routine's id"), PeriodQuery),
    responses(
        (status = 200, description = "Occurrences and statistics", body = RoutineOccurrences),
        (status = 400, description = "Bad zone, date or length", body = ProblemBody),
        (status = 404, description = "No such Routine (`routine_not_found`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_occurrences(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<PeriodQuery>,
) -> Result<Json<RoutineOccurrences>, TimeError> {
    let period = query.period()?;
    let routine = routines::get(&state.pool, id)
        .await?
        .ok_or(RoutineError::NotFound)?;
    let entries = load(&state.pool, &period).await?;
    let found = occurrences(&period, &routine.steps, &entries);
    let figures = routine_stats(period.tz(), routine.steps.len(), &found);
    tracing::debug!(
        routine_id = id,
        occurrences = found.len(),
        "routine occurrences found"
    );
    Ok(Json(RoutineOccurrences {
        routine,
        occurrences: found,
        stats: figures,
    }))
}
