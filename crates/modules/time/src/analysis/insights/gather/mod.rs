//! Turns stored rows into the [`Inputs`] the Insight kinds read: one measured series per
//! Routine Step, Routine total, frequent description, top app and top Context.
//!
//! Everything is measured over a *reach* that extends the Period back far enough to fill the
//! earlier change window, so Insights about a short Period (even one Day) can still compare the
//! last four weeks with the four before.

mod descriptions;
mod measure;
mod routines;
mod screen;

use std::collections::HashSet;

use chrono::{DateTime, Days, NaiveDate, Utc};

use super::inputs::{CHANGE_WINDOW_DAYS, DateRange, Inputs};
use crate::analysis::{
    Period, PeriodError,
    activity::ActivityRow,
    lifelog::{LifeEntry, fold},
    routines::{MatchKind, Routine},
};

/// The stored rows Insights are found in.
pub struct Sources<'a> {
    /// Time Entries touching the reach.
    pub entries: &'a [LifeEntry],
    /// Activity touching the reach.
    pub activity: &'a [ActivityRow],
    /// Every saved Routine.
    pub routines: &'a [Routine],
    /// The present moment: a Day that has not finished is left out of screen time.
    pub now: DateTime<Utc>,
}

/// The Period to load rows for: the report's Days, extended back to cover both change windows.
///
/// # Errors
/// A [`PeriodError`] cannot normally happen (the reach is at most as long as the longer of the
/// report and two windows), but is returned rather than hidden.
pub fn reach_period(report: &Period) -> Result<Period, PeriodError> {
    let last = report.days().last().map_or(NaiveDate::MIN, |d| d.date);
    let first = report.days().first().map_or(last, |d| d.date);
    let windows_start = last
        .checked_sub_days(Days::new(2 * CHANGE_WINDOW_DAYS - 1))
        .unwrap_or(first);
    Period::parse(
        &first.min(windows_start).to_string(),
        &last.to_string(),
        report.tz().name(),
    )
}

/// The three windows of Days for a report ending on `last`: the report, the last four weeks
/// and the four weeks before.
#[must_use]
pub fn windows(report: &Period) -> (DateRange, DateRange, DateRange) {
    let last = report.days().last().map_or(NaiveDate::MIN, |d| d.date);
    let first = report.days().first().map_or(last, |d| d.date);
    let back = |days: u64| {
        last.checked_sub_days(Days::new(days))
            .unwrap_or(NaiveDate::MIN)
    };
    (
        DateRange {
            from: first,
            to: last,
        },
        DateRange {
            from: back(CHANGE_WINDOW_DAYS - 1),
            to: last,
        },
        DateRange {
            from: back(2 * CHANGE_WINDOW_DAYS - 1),
            to: back(CHANGE_WINDOW_DAYS),
        },
    )
}

/// Everything measured in `sources`, with the report and change windows.
#[must_use]
pub fn gather(report: &Period, reach: &Period, sources: &Sources) -> Inputs {
    let (period, recent, earlier) = windows(report);
    let covered: HashSet<String> = sources
        .routines
        .iter()
        .flat_map(|r| &r.steps)
        .filter(|step| step.kind == MatchKind::Description)
        .map(|step| fold(&step.value))
        .collect();
    let mut series = routines::series(reach, sources.routines, sources.entries);
    series.extend(descriptions::series(
        report,
        reach,
        sources.entries,
        &covered,
    ));
    series.extend(screen::series(report, reach, sources.activity, sources.now));
    Inputs {
        report: period,
        recent,
        earlier,
        series,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(text: &str) -> chrono::NaiveDate {
        text.parse().unwrap()
    }

    #[test]
    fn a_short_period_reaches_back_55_days_before_its_last_day() {
        let report = Period::parse("2026-05-30", "2026-05-31", "UTC").unwrap();
        let reach = reach_period(&report).unwrap();
        assert_eq!(reach.days().first().unwrap().date, d("2026-04-06"));
        assert_eq!(reach.days().last().unwrap().date, d("2026-05-31"));
    }

    #[test]
    fn a_long_period_keeps_its_own_first_day() {
        let report = Period::parse("2026-01-01", "2026-05-31", "UTC").unwrap();
        assert_eq!(
            reach_period(&report).unwrap().days().first().unwrap().date,
            d("2026-01-01")
        );
    }

    #[test]
    fn the_windows_are_the_period_and_the_two_four_week_stretches_before_its_end() {
        let report = Period::parse("2026-05-30", "2026-05-31", "UTC").unwrap();
        let (period, recent, earlier) = windows(&report);
        assert_eq!((period.from, period.to), (d("2026-05-30"), d("2026-05-31")));
        assert_eq!((recent.from, recent.to), (d("2026-05-04"), d("2026-05-31")));
        assert_eq!(
            (earlier.from, earlier.to),
            (d("2026-04-06"), d("2026-05-03"))
        );
    }
}
