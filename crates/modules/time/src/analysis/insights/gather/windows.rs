//! The Days Insights look at: the report, and the two windows a change is measured between.
//!
//! Only finished Days count, so every window ends on the report's last Day or the last finished
//! Day, whichever is earlier. A Day still going (or yet to come) is partial and would drag
//! averages and medians down.

use chrono::{DateTime, Days, NaiveDate, Utc};

use crate::analysis::{
    Period, PeriodError,
    insights::inputs::{CHANGE_WINDOW_DAYS, DateRange},
    last_finished_date,
};

/// The three stretches of Days one request looks at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Windows {
    /// The report's Days that have finished (empty when none have).
    pub report: DateRange,
    /// The last [`CHANGE_WINDOW_DAYS`] finished Days up to the report's end.
    pub recent: DateRange,
    /// The [`CHANGE_WINDOW_DAYS`] Days before `recent`.
    pub earlier: DateRange,
}

/// The windows for a report seen at `now`.
#[must_use]
pub fn windows(report: &Period, now: DateTime<Utc>) -> Windows {
    let through = last_finished_date(report.tz(), now);
    let first = report.days().first().map_or(NaiveDate::MIN, |d| d.date);
    let last = report
        .days()
        .last()
        .map_or(NaiveDate::MIN, |d| d.date)
        .min(through);
    let back = |days: u64| {
        last.checked_sub_days(Days::new(days))
            .unwrap_or(NaiveDate::MIN)
    };
    Windows {
        report: DateRange {
            from: first,
            to: last,
        },
        recent: DateRange {
            from: back(CHANGE_WINDOW_DAYS - 1),
            to: last,
        },
        earlier: DateRange {
            from: back(2 * CHANGE_WINDOW_DAYS - 1),
            to: back(CHANGE_WINDOW_DAYS),
        },
    }
}

/// The Period to load rows for: every Day the windows cover.
///
/// # Errors
/// A [`PeriodError`] cannot normally happen (the reach is at most as long as the longer of the
/// report and two windows), but is returned rather than hidden.
pub fn reach_period(report: &Period, now: DateTime<Utc>) -> Result<Period, PeriodError> {
    let all = windows(report, now);
    Period::parse(
        &all.report.from.min(all.earlier.from).to_string(),
        &all.recent.to.to_string(),
        report.tz().name(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(text: &str) -> NaiveDate {
        text.parse().unwrap()
    }

    fn now(text: &str) -> DateTime<Utc> {
        text.parse().unwrap()
    }

    fn report(from: &str, to: &str) -> Period {
        Period::parse(from, to, "UTC").unwrap()
    }

    fn range(r: DateRange) -> (String, String) {
        (r.from.to_string(), r.to.to_string())
    }

    const JUNE: &str = "2026-06-01T12:00:00Z";

    #[test]
    fn the_windows_are_the_period_and_the_two_four_week_stretches_before_its_end() {
        let all = windows(&report("2026-05-30", "2026-05-31"), now(JUNE));
        assert_eq!(
            range(all.report),
            ("2026-05-30".into(), "2026-05-31".into())
        );
        assert_eq!(
            range(all.recent),
            ("2026-05-04".into(), "2026-05-31".into())
        );
        assert_eq!(
            range(all.earlier),
            ("2026-04-06".into(), "2026-05-03".into())
        );
    }

    #[test]
    fn a_period_reaching_into_the_future_ends_every_window_on_the_last_finished_day() {
        // June 1 is still going at noon, so May 31 is the last finished Day.
        let all = windows(&report("2026-05-25", "2026-06-07"), now(JUNE));
        assert_eq!(
            range(all.report),
            ("2026-05-25".into(), "2026-05-31".into())
        );
        assert_eq!(range(all.recent).1, "2026-05-31");
        assert_eq!(
            range(all.earlier),
            ("2026-04-06".into(), "2026-05-03".into())
        );
    }

    #[test]
    fn today_alone_has_no_finished_day_but_still_compares_the_weeks_before() {
        let all = windows(&report("2026-06-01", "2026-06-01"), now(JUNE));
        assert!(all.report.from > all.report.to);
        assert_eq!(range(all.recent).1, "2026-05-31");
    }

    #[test]
    fn the_reach_covers_the_report_and_both_windows_and_nothing_after() {
        let reach = reach_period(&report("2026-05-30", "2026-06-07"), now(JUNE)).unwrap();
        assert_eq!(reach.days().first().unwrap().date, d("2026-04-06"));
        assert_eq!(reach.days().last().unwrap().date, d("2026-05-31"));
    }

    #[test]
    fn a_long_period_keeps_its_own_first_day() {
        let reach = reach_period(&report("2026-01-01", "2026-05-31"), now(JUNE)).unwrap();
        assert_eq!(reach.days().first().unwrap().date, d("2026-01-01"));
    }
}
