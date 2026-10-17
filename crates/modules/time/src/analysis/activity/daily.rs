//! Screen time per finished Day: the figures an average per Day is made of.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;

use super::{ActivityRow, seconds};
use crate::analysis::{DayPortion, Period, split_by_day};

/// One Day's screen time.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct DayTotal {
    /// Local date.
    #[schema(value_type = String, format = Date)]
    pub date: NaiveDate,
    /// Seconds of Activity on it.
    pub seconds: i64,
}

/// Screen time of each Day of the Period that has Activity and is finished (on or before
/// `through`), earliest first. A Day not yet over is partial, and a Day with no Activity usually
/// has no import, so neither stands for a typical Day. The one rule for what counts as an
/// elapsed Day: Insights and the Overview comparison both use it.
#[must_use]
pub fn day_totals(period: &Period, rows: &[ActivityRow], through: NaiveDate) -> Vec<DayTotal> {
    let mut millis: BTreeMap<NaiveDate, i64> = BTreeMap::new();
    for row in rows {
        for portion in split_by_day(period, row.start, row.end) {
            *millis.entry(portion.date).or_default() += DayPortion::millis(&portion);
        }
    }
    millis
        .into_iter()
        .filter(|(date, ms)| *date <= through && *ms > 0)
        .map(|(date, ms)| DayTotal {
            date,
            seconds: seconds(ms),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(from: &str, to: &str) -> ActivityRow {
        ActivityRow::new("Brave", "", from.parse().unwrap(), to.parse().unwrap())
    }

    fn period() -> Period {
        Period::parse("2026-05-01", "2026-05-10", "UTC").unwrap()
    }

    fn found(through: &str, rows: &[ActivityRow]) -> Vec<(String, i64)> {
        day_totals(&period(), rows, through.parse().unwrap())
            .into_iter()
            .map(|d| (d.date.to_string(), d.seconds))
            .collect()
    }

    #[test]
    fn each_day_with_activity_has_its_seconds() {
        let rows = [
            row("2026-05-04T09:00:00Z", "2026-05-04T10:00:00Z"),
            row("2026-05-04T11:00:00Z", "2026-05-04T11:30:00Z"),
            row("2026-05-06T09:00:00Z", "2026-05-06T09:10:00Z"),
        ];
        assert_eq!(
            found("2026-05-10", &rows),
            [
                ("2026-05-04".to_owned(), 5400),
                ("2026-05-06".to_owned(), 600)
            ]
        );
    }

    #[test]
    fn days_after_the_last_finished_one_are_left_out() {
        let rows = [
            row("2026-05-04T09:00:00Z", "2026-05-04T10:00:00Z"),
            row("2026-05-05T09:00:00Z", "2026-05-05T10:00:00Z"),
        ];
        assert_eq!(
            found("2026-05-04", &rows),
            [("2026-05-04".to_owned(), 3600)]
        );
    }

    #[test]
    fn a_row_over_midnight_counts_toward_both_days() {
        let rows = [row("2026-05-04T23:00:00Z", "2026-05-05T01:00:00Z")];
        assert_eq!(
            found("2026-05-10", &rows),
            [
                ("2026-05-04".to_owned(), 3600),
                ("2026-05-05".to_owned(), 3600)
            ]
        );
    }
}
