//! The Period, its headline numbers and its per-Day totals.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::report::period::PeriodKind;

/// The Period covered and the Usual's range it is compared with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct TimelinePeriod {
    /// How the Period was chosen.
    pub kind: PeriodKind,
    /// First Day.
    pub from: NaiveDate,
    /// Last Day (inclusive).
    pub to: NaiveDate,
    /// First Day of the Usual's range.
    pub usual_from: NaiveDate,
    /// Last Day of the Usual's range (inclusive).
    pub usual_to: NaiveDate,
    /// Days of the Period up to and including today; all of them once it is past.
    pub elapsed_days: i64,
    /// Days of the Usual's range on or after the Day of the first Segment.
    pub usual_available_days: i64,
}

/// Headline numbers of the Period.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, ToSchema)]
pub struct TimelineOverview {
    /// Seconds known not to be Away.
    pub active_secs: i64,
    /// Seconds known to be Away.
    pub away_secs: i64,
    /// Seconds whose Away is unknown. Never counted as Active.
    pub unknown_secs: i64,
    /// Switches between apps.
    pub switches: i64,
    /// Switches per Active hour; absent with no Active time.
    pub switches_per_active_hour: Option<f64>,
    /// The longest Focus stretch, in seconds.
    pub longest_focus_secs: i64,
    /// Share (0 to 1) of Active time spent in Focus stretches of 25 minutes or more.
    pub focus_share: f64,
    /// Start of the first Active moment in the Period.
    pub first_active: Option<DateTime<Utc>>,
    /// End of the last Active moment in the Period.
    pub last_active: Option<DateTime<Utc>>,
}

/// Seconds on one Day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct DayTotals {
    /// The local Day.
    pub date: NaiveDate,
    /// Active seconds.
    pub active_secs: i64,
    /// Away seconds.
    pub away_secs: i64,
    /// Seconds whose Away is unknown.
    pub unknown_secs: i64,
}
