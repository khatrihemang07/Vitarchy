//! Analysis of Time Entries and Activity: pure functions over rows (ADR 0010).

pub mod day_split;
pub mod period;

pub use day_split::{DayPortion, split_by_day};
pub use period::{Day, MAX_DAYS, Period, PeriodError};
