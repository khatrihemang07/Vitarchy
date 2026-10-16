//! Life log analyses: pure aggregations of Time Entries over a Period.

mod entry;
mod split;
mod spread;
mod stats;
mod timeline;
mod totals;

pub use entry::{LifeEntry, NO_DESCRIPTION, NO_PROJECT};
pub use split::{DaySplit, ProjectSeconds, day_splits};
pub use spread::{ClockSpread, DurationSpread, clock_spread, duration_spread};
pub use stats::{DescriptionStat, MIN_DAYS, description_stats};
pub use timeline::{LifeLogSpan, timeline};
pub use totals::{DescriptionTotal, LifeLogTotals, ProjectTotal, totals};
