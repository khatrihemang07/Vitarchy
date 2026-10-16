//! `GET /api/time/analysis/period`: the local Days a Period covers.

use axum::{Json, extract::Query};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::params::PeriodQuery;
use crate::{
    analysis::{Day, Period},
    error::{ProblemBody, TimeError},
};

/// One local Day and where it begins and ends.
#[derive(Serialize, ToSchema)]
pub struct DayBounds {
    /// Local date.
    #[schema(value_type = String, format = Date)]
    pub date: NaiveDate,
    /// Start of the Day (inclusive).
    pub start: DateTime<Utc>,
    /// End of the Day (exclusive).
    pub end: DateTime<Utc>,
}

/// The Days of a Period.
#[derive(Serialize, ToSchema)]
pub struct PeriodDays {
    /// The IANA zone used.
    pub tz: String,
    /// Days, earliest first.
    pub days: Vec<DayBounds>,
}

impl From<&Period> for PeriodDays {
    fn from(period: &Period) -> Self {
        Self {
            tz: period.tz().name().to_owned(),
            days: period.days().iter().map(DayBounds::from).collect(),
        }
    }
}

impl From<&Day> for DayBounds {
    fn from(day: &Day) -> Self {
        Self {
            date: day.date,
            start: day.start,
            end: day.end,
        }
    }
}

/// Resolves `from`, `to` and `tz` into local Days (also validates them).
#[utoipa::path(
    get,
    path = "/analysis/period",
    tag = "time",
    params(PeriodQuery),
    responses(
        (status = 200, description = "The Days of the Period", body = PeriodDays),
        (status = 400, description = "Bad zone, date or length (`unknown_time_zone`, `invalid_date`, `period_reversed`, `period_too_long`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_period(Query(query): Query<PeriodQuery>) -> Result<Json<PeriodDays>, TimeError> {
    let period = query.period()?;
    tracing::debug!(
        days = period.days().len(),
        tz = period.tz().name(),
        "period resolved"
    );
    Ok(Json(PeriodDays::from(&period)))
}
