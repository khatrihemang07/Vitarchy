//! The Timeline summary as the API returns it, one file per concern.

mod apps;
mod lists;
mod period;
mod usual;

use serde::Serialize;
use utoipa::ToSchema;

pub use apps::{AppUsage, SiteUsage, TitleUsage};
pub use lists::{LaneEntry, SegmentView};
pub use period::{DayTotals, TimelineOverview, TimelinePeriod};
pub use usual::{UsualFigure, UsualTimeline};

/// The Timeline of one Period, analysed.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TimelineSummary {
    /// The Period covered.
    pub period: TimelinePeriod,
    /// Headline numbers.
    pub overview: TimelineOverview,
    /// The Usual; absent when there is nothing to compare with.
    pub usual: Option<UsualTimeline>,
    /// One entry per Day of the Period.
    pub days: Vec<DayTotals>,
    /// Active seconds by local hour of day, 24 values starting at midnight.
    pub hours: Vec<i64>,
    /// Apps, most Active time first.
    pub apps: Vec<AppUsage>,
    /// The Period's Segments; only for a Day.
    pub segments: Option<Vec<SegmentView>>,
    /// The Day's Time Entries; only for a Day.
    pub entries: Option<Vec<LaneEntry>>,
}
