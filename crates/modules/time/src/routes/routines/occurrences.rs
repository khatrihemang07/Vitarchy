//! `GET /api/time/routines/occurrences`: every saved Routine over a Period, from one load.

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Serialize;
use utoipa::ToSchema;
use vitarchy_core::AppState;

use crate::{
    analysis::routines::{Occurrence, Routine, RoutineStats, occurrences, routine_stats},
    error::{ProblemBody, TimeError},
    repo::routines,
    routes::analysis::{lifelog::load_for_routines, params::PeriodQuery},
};

/// How one Routine went over a Period.
#[derive(Serialize, ToSchema)]
pub struct RoutineOccurrences {
    /// The Routine.
    pub routine: Routine,
    /// Each Day it happened, earliest first.
    pub occurrences: Vec<Occurrence>,
    /// The usual times and lengths.
    pub stats: RoutineStats,
}

/// Every saved Routine over a Period.
#[derive(Serialize, ToSchema)]
pub struct AllRoutineOccurrences {
    /// By Routine name.
    pub routines: Vec<RoutineOccurrences>,
}

/// Every Routine's Occurrences, complete and incomplete, with statistics. The Time Entries are
/// loaded once however many Routines there are.
#[utoipa::path(
    get,
    path = "/routines/occurrences",
    tag = "time",
    params(PeriodQuery),
    responses(
        (status = 200, description = "Each Routine with its Occurrences and statistics", body = AllRoutineOccurrences),
        (status = 400, description = "Bad zone, date or length", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_occurrences(
    State(state): State<AppState>,
    Query(query): Query<PeriodQuery>,
) -> Result<Json<AllRoutineOccurrences>, TimeError> {
    let period = query.period()?;
    let (saved, entries) = tokio::try_join!(
        routines::list(&state.pool),
        load_for_routines(&state.pool, &period)
    )?;
    let found: Vec<RoutineOccurrences> = saved
        .into_iter()
        .map(|routine| {
            let days = occurrences(&period, &routine.steps, &entries);
            let figures = routine_stats(period.tz(), routine.steps.len(), &days);
            RoutineOccurrences {
                routine,
                occurrences: days,
                stats: figures,
            }
        })
        .collect();
    tracing::debug!(routines = found.len(), "routine occurrences found");
    Ok(Json(AllRoutineOccurrences { routines: found }))
}
