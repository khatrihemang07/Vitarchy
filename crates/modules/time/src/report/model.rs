//! The Report as the API returns it.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::{
    compare::Comparison, grouping::Grouping, period::PeriodKind, stats::Stats,
    usual::UsualComparison,
};

/// Seconds tracked on one Day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct DailyPoint {
    /// The local Day.
    pub date: NaiveDate,
    /// Seconds tracked that Day; 0 when nothing was.
    pub seconds: i64,
}

/// A Time Entry listed under a bottom-level group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ReportEntry {
    /// Toggl's id for the entry.
    pub toggl_id: i64,
    /// When it really started (may be before the Period).
    pub start: DateTime<Utc>,
    /// When it really stopped (may be after the Period).
    pub stop: DateTime<Utc>,
    /// Seconds inside the Period only: an entry crossing the Period's edge is clipped.
    pub duration_s: i64,
    /// What the person typed; may be empty.
    pub description: String,
    /// The project's id, when it has one.
    pub project_id: Option<i64>,
    /// The project's name, when known.
    pub project_name: Option<String>,
    /// Tag names.
    pub tags: Vec<String>,
}

/// One row of the summary: a description, project or tag, with its numbers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Group {
    /// Stable identity within its level; empty for the "(no ...)" group.
    pub key: String,
    /// What to show: the most common spelling of the group.
    pub label: String,
    /// Days, entries, total and the per-Day and per-entry spreads.
    pub stats: Stats,
    /// Comparison with the same group in the Previous Period.
    pub vs_previous: Comparison,
    /// Comparison with the same group's Usual (the Usual's range).
    pub vs_usual: UsualComparison,
    /// One point per Day of the Period, 0 where the group has no time.
    pub daily: Vec<DailyPoint>,
    /// Second-level groups, when a second grouping was asked for. Empty otherwise.
    #[schema(no_recursion)]
    pub children: Vec<Group>,
    /// The entries, only on bottom-level groups (empty on groups that have children).
    pub entries: Vec<ReportEntry>,
}

/// Change of the overview figures from a reference (current minus reference).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, ToSchema)]
pub struct OverviewChange {
    /// Seconds.
    pub total_s: i64,
    /// Count.
    pub entries: i64,
    /// Count.
    pub days_tracked: i64,
    /// Seconds.
    pub avg_per_day_s: i64,
}

/// The Period's headline numbers. An entry with several tags is counted once here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct Overview {
    /// Seconds tracked.
    pub total_s: i64,
    /// Distinct Time Entries.
    pub entries: i64,
    /// Days with any time.
    pub days_tracked: i64,
    /// Total divided by the Days tracked, rounded; 0 when none.
    pub avg_per_day_s: i64,
    /// Days with entries in the Usual's range (the Usual's range).
    pub usual_days: i64,
    /// Change from the Previous Period.
    pub vs_previous: OverviewChange,
    /// Change from the Usual, whose total, entries and Days tracked are scaled to the
    /// Days that have passed (see [`PeriodInfo::elapsed_days`]); the per-Day average is not.
    pub vs_usual: OverviewChange,
}

/// The Period the Report covers, the Previous Period and the Usual's range it is compared with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct PeriodInfo {
    /// How the Period was chosen.
    pub kind: PeriodKind,
    /// First Day.
    pub from: NaiveDate,
    /// Last Day (inclusive).
    pub to: NaiveDate,
    /// First Day of the Previous Period.
    pub previous_from: NaiveDate,
    /// Last Day of the Previous Period (inclusive).
    pub previous_to: NaiveDate,
    /// First Day of the Usual's range (the Days ending the Day before `from`).
    pub usual_from: NaiveDate,
    /// Last Day of the Usual's range (inclusive).
    pub usual_to: NaiveDate,
    /// Days of the Period up to and including today (in the request's time zone); all of
    /// them once the Period is past, 0 before it starts.
    pub elapsed_days: i64,
    /// Days of the Usual's range on or after the Day of the first Time Entry ever tracked;
    /// fewer than the range holds when the history is short, 0 when there is none yet.
    pub usual_available_days: i64,
}

/// A summary of the Time Entries in one Period, compared with the Previous Period
/// and with the Usual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Report {
    /// The Period covered.
    pub period: PeriodInfo,
    /// Headline numbers.
    pub overview: Overview,
    /// One point per Day of the Period.
    pub daily: Vec<DailyPoint>,
    /// The first-level grouping used.
    pub group1: Grouping,
    /// The second-level grouping used, when there is one.
    pub group2: Option<Grouping>,
    /// First-level groups, largest total first.
    pub groups: Vec<Group>,
}
