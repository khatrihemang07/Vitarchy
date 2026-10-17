//! Activity analyses: pure aggregations of [`ActivityRow`]s over a Period.
//! Activity is counted raw (no idle correction) and split at local midnights.

mod bounds;
mod busiest;
mod daily;
mod heatmap;
mod row;
mod timeline;
mod totals;
mod trend;

pub use bounds::{DayBound, day_bounds};
pub use busiest::{RankedContext, TOP_SUBJECTS, top_contexts};
pub use daily::{DayTotal, day_totals};
pub use heatmap::{HOURS, Heatmap, WEEKDAYS, heatmap};
pub use row::ActivityRow;
pub use timeline::{MERGE_GAP_SECS, Span, timeline};
pub use totals::{AppTotal, ContextTotal, TOP_TITLES, TitleTotal, Totals, totals};
pub use trend::{TOP_APPS, Trend, TrendSeries, TrendWeek, WeekStart, trend};

/// Whole seconds from milliseconds, rounded: sums are kept in milliseconds so
/// rounding happens once, not once per row.
fn seconds(millis: i64) -> i64 {
    (millis + 500) / 1000
}
