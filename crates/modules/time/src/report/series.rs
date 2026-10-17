//! Daily totals across a Period.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use super::{day_split::DayPart, model::DailyPoint};

/// One point for each of `days`, with 0 where no part falls.
#[must_use]
pub fn daily_series<'a>(
    days: &[NaiveDate],
    parts: impl Iterator<Item = &'a DayPart>,
) -> Vec<DailyPoint> {
    let mut totals: BTreeMap<NaiveDate, i64> = BTreeMap::new();
    for part in parts {
        *totals.entry(part.date).or_default() += part.seconds;
    }
    days.iter()
        .map(|date| DailyPoint {
            date: *date,
            seconds: totals.get(date).copied().unwrap_or(0),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
    }

    #[test]
    fn every_day_has_a_point_and_gaps_are_zero() {
        let parts = [
            DayPart {
                date: d(2),
                seconds: 60,
            },
            DayPart {
                date: d(2),
                seconds: 30,
            },
        ];
        let series = daily_series(&[d(1), d(2), d(3)], parts.iter());
        let seconds: Vec<i64> = series.iter().map(|p| p.seconds).collect();
        assert_eq!(seconds, [0, 90, 0]);
    }
}
