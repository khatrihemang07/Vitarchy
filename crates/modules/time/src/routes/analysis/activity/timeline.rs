//! `GET /api/time/analysis/activity/timeline`.

use axum::{
    Json,
    extract::{Query, State},
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use vitarchy_core::AppState;

use super::load;
use crate::{
    analysis::{
        Period,
        activity::{Span, timeline},
    },
    error::{ProblemBody, TimeError},
};

/// One Day, asked for by the browser.
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DayQuery {
    /// The local date, `YYYY-MM-DD`.
    #[serde(default)]
    pub day: String,
    /// IANA time zone the Day is measured in.
    #[serde(default)]
    pub tz: String,
}

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
    let period = Period::parse(&query.day, &query.day, &query.tz)?;
    let spans = timeline(&period, &load(&state.pool, &period).await?);
    let day = period.days()[0];
    Ok(Json(DayTimeline {
        date: day.date,
        day_start: day.start,
        day_end: day.end,
        spans,
    }))
}
