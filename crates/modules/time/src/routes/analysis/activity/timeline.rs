//! `GET /api/time/analysis/activity/timeline`.

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
    analysis::activity::{Span, timeline},
    error::{ProblemBody, TimeError},
    routes::analysis::params::DayQuery,
};

/// One Day's lane of spans.
#[derive(Serialize, ToSchema)]
pub struct DayTimeline {
    /// Local date.
    #[schema(value_type = String, format = Date)]
    pub date: NaiveDate,
    /// Local midnight that begins the Day.
    pub day_start: DateTime<Utc>,
    /// Local midnight that ends the Day.
    pub day_end: DateTime<Utc>,
    /// Spans in time order.
    pub spans: Vec<Span>,
}

/// The Day's Activity as merged spans by Context.
#[utoipa::path(
    get,
    path = "/analysis/activity/timeline",
    tag = "time",
    params(DayQuery),
    responses(
        (status = 200, description = "The Day's spans", body = DayTimeline),
        (status = 400, description = "Bad zone or date", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_timeline(
    State(state): State<AppState>,
    Query(query): Query<DayQuery>,
) -> Result<Json<DayTimeline>, TimeError> {
    let period = query.period()?;
    let spans = timeline(&period, &load(&state.pool, &period).await?);
    let day = period.days()[0];
    Ok(Json(DayTimeline {
        date: day.date,
        day_start: day.start,
        day_end: day.end,
        spans,
    }))
}
