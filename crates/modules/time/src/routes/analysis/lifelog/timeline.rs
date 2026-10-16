//! `GET /api/time/analysis/lifelog/timeline`.

use axum::{
    Json,
    extract::{Query, State},
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use vitarchy_core::AppState;

use super::load;
use crate::{
    analysis::lifelog::{LifeLogSpan, timeline},
    error::{ProblemBody, TimeError},
    routes::analysis::params::DayQuery,
};

/// One Day's lane of Time Entries.
#[derive(Serialize, ToSchema)]
pub struct LifeLogTimeline {
    /// Local date.
    #[schema(value_type = String, format = Date)]
    pub date: NaiveDate,
    /// Local midnight that begins the Day.
    pub day_start: DateTime<Utc>,
    /// Local midnight that ends the Day.
    pub day_end: DateTime<Utc>,
    /// Entries in time order, clipped to the Day.
    pub spans: Vec<LifeLogSpan>,
}

/// The Day's Time Entries with their project colour and description.
#[utoipa::path(
    get,
    path = "/analysis/lifelog/timeline",
    tag = "time",
    params(DayQuery),
    responses(
        (status = 200, description = "The Day's entries", body = LifeLogTimeline),
        (status = 400, description = "Bad zone or date", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_lifelog_timeline(
    State(state): State<AppState>,
    Query(query): Query<DayQuery>,
) -> Result<Json<LifeLogTimeline>, TimeError> {
    let period = query.period()?;
    let spans = timeline(&period, &load(&state.pool, &period).await?);
    let day = period.days()[0];
    Ok(Json(LifeLogTimeline {
        date: day.date,
        day_start: day.start,
        day_end: day.end,
        spans,
    }))
}
