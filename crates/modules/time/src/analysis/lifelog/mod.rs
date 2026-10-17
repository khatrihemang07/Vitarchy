//! Life log analyses: pure aggregations of Time Entries over a Period.

mod entry;
mod split;
mod spread;
mod stats;
mod timeline;
mod totals;

pub use entry::{LifeEntry, NO_DESCRIPTION, NO_PROJECT, fold};
pub use split::{DaySplit, ProjectSeconds, day_splits};
pub use spread::{ClockSpread, DurationSpread, clock_spread, duration_spread, quantile};
pub use stats::{DescriptionStat, MIN_DAYS, attributed_days, description_stats, minute_of_day};
pub use timeline::{LifeLogSpan, timeline};
pub use totals::{DescriptionTotal, LifeLogTotals, ProjectTotal, totals};
