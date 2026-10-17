//! How the Usual is scaled to what a Period can fairly be compared with.
//!
//! A Period still in progress has only some of its Days behind it, and the person's history
//! may start partway through the Usual's range. Scaling the Usual's totals and counts to the
//! Days that have passed, against the Days of history available, keeps both honest.

use chrono::{DateTime, NaiveDate, Utc};

use super::{
    model::Overview,
    period::{Period, USUAL_DAYS},
    stats::Stats,
};

/// The two Day counts a Usual is scaled by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scaling {
    /// Days of the Period up to and including today; all of them once the Period is past.
    pub elapsed_days: i64,
    /// Days of the Usual's range on or after the Day of the person's first Time Entry.
    pub available_days: i64,
}

/// The figures of a Usual that grow with the number of Days: seconds, entries and Days.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Totals {
    /// Seconds.
    pub total_s: i64,
    /// Count.
    pub entries: i64,
    /// Count.
    pub days: i64,
}

impl From<&Stats> for Totals {
    fn from(stats: &Stats) -> Self {
        Self {
            total_s: stats.total_s,
            entries: stats.entries,
            days: stats.days,
        }
    }
}

impl From<&Overview> for Totals {
    fn from(overview: &Overview) -> Self {
        Self {
            total_s: overview.total_s,
            entries: overview.entries,
            days: overview.days_tracked,
        }
    }
}

fn local_day(period: &Period, moment: DateTime<Utc>) -> NaiveDate {
    moment.with_timezone(&period.tz).date_naive()
}

fn clamp_days(days: i64, most: i64) -> i64 {
    days.clamp(0, most)
}

impl Scaling {
    /// Reads the Day counts for `period` at `now`, with `first_entry` the start of the
    /// person's earliest Time Entry. Both are turned into local Days in the Period's time zone.
    #[must_use]
    pub fn new(period: &Period, now: DateTime<Utc>, first_entry: Option<DateTime<Utc>>) -> Self {
        let length = (period.to - period.from).num_days() + 1;
        let elapsed_days = clamp_days(
            (local_day(period, now) - period.from).num_days() + 1,
            length,
        );
        let usual = period.usual_range();
        let available_days = first_entry.map_or(0, |first| {
            let from = local_day(period, first).max(usual.from);
            clamp_days((usual.to - from).num_days() + 1, USUAL_DAYS)
        });
        Self {
            elapsed_days,
            available_days,
        }
    }

    /// False when there is nothing to compare: the Period has not started, or there is no
    /// history in the Usual's range yet.
    #[must_use]
    pub fn is_comparable(&self) -> bool {
        self.elapsed_days > 0 && self.available_days > 0
    }

    fn scale(&self, value: i64) -> i64 {
        if self.available_days == 0 {
            return 0;
        }
        (value * self.elapsed_days + self.available_days / 2) / self.available_days
    }

    /// The Usual's totals for the Days that have passed: each figure times elapsed Days over
    /// available Days, rounding half up.
    #[must_use]
    pub fn apply(&self, usual: Totals) -> Totals {
        Totals {
            total_s: self.scale(usual.total_s),
            entries: self.scale(usual.entries),
            days: self.scale(usual.days),
        }
    }
}
