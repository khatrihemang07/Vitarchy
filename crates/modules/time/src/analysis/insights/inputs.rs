//! What every Insight kind reads: one list of measured series and the three windows of Days.

use chrono::NaiveDate;

use super::insight::{Measure, Subject};
use crate::analysis::lifelog::MIN_DAYS;

/// Fewest Days a typical value, a weekday comparison or an outlier needs: the Life log's habit
/// threshold, so a description is a habit in both places.
pub const MIN_SAMPLES: usize = MIN_DAYS;

/// An inclusive stretch of local dates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateRange {
    /// First date.
    pub from: NaiveDate,
    /// Last date.
    pub to: NaiveDate,
}

impl DateRange {
    /// Whether `date` lies inside.
    #[must_use]
    pub fn contains(self, date: NaiveDate) -> bool {
        (self.from..=self.to).contains(&date)
    }
}

/// One measurement on one Day.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// The local date it belongs to.
    pub date: NaiveDate,
    /// Minutes: a time of day, or a length.
    pub value: f64,
}

/// Every measurement of one quantity of one subject.
#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    /// What is measured.
    pub subject: Subject,
    /// Which quantity.
    pub measure: Measure,
    /// The measurements, in no particular order.
    pub points: Vec<Point>,
}

impl Series {
    /// The values measured inside `range`.
    #[must_use]
    pub fn values_in(&self, range: DateRange) -> Vec<f64> {
        self.points
            .iter()
            .filter(|p| range.contains(p.date))
            .map(|p| p.value)
            .collect()
    }

    /// The points inside `range`.
    #[must_use]
    pub fn points_in(&self, range: DateRange) -> Vec<Point> {
        self.points
            .iter()
            .filter(|p| range.contains(p.date))
            .copied()
            .collect()
    }

    /// Whether the values are times of day (which wrap at midnight) rather than lengths.
    #[must_use]
    pub fn is_clock(&self) -> bool {
        matches!(self.measure, Measure::Start | Measure::End)
    }
}

/// The data the Insight kinds work from.
#[derive(Debug, Clone, PartialEq)]
pub struct Inputs {
    /// The Period being looked at: typical values, weekdays and outliers use its Days.
    pub report: DateRange,
    /// The last [`CHANGE_WINDOW_DAYS`] Days ending on the Period's last Day.
    pub recent: DateRange,
    /// The [`CHANGE_WINDOW_DAYS`] Days before `recent`.
    pub earlier: DateRange,
    /// Everything measured, over Days that reach back far enough to fill `earlier`.
    pub series: Vec<Series>,
}

/// Length of each window a change is measured over: four weeks.
pub const CHANGE_WINDOW_DAYS: u64 = 28;

#[cfg(test)]
pub(super) mod fixtures {
    use super::*;
    use crate::analysis::insights::insight::SubjectKind;

    /// 2026-05-`day`.
    pub fn d(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 5, day).unwrap()
    }

    /// A series on a routine step, one point per `(day, value)`.
    pub fn series(measure: Measure, points: &[(u32, f64)]) -> Series {
        Series {
            subject: Subject {
                kind: SubjectKind::RoutineStep,
                name: "office".into(),
                routine: None,
            },
            measure,
            points: points
                .iter()
                .map(|(day, value)| Point {
                    date: d(*day),
                    value: *value,
                })
                .collect(),
        }
    }

    /// Windows covering May 1-31, with `recent` = May 4-31 and `earlier` = April 6 to May 3.
    pub fn inputs(series: Vec<Series>) -> Inputs {
        Inputs {
            report: DateRange {
                from: d(1),
                to: d(31),
            },
            recent: DateRange {
                from: d(4),
                to: d(31),
            },
            earlier: DateRange {
                from: NaiveDate::from_ymd_opt(2026, 4, 6).unwrap(),
                to: d(3),
            },
            series,
        }
    }
}
