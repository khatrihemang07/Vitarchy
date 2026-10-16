//! Analysis of Time Entries and Activity: pure functions over rows (ADR 0010).

pub mod activity;
pub mod context;
mod context_rules;
pub mod day_split;
pub mod insights;
pub mod lifelog;
pub mod period;
pub mod routines;

pub use day_split::{DayPortion, split_by_day};
pub use period::{Day, MAX_DAYS, Period, PeriodError};
