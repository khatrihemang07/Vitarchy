//! When each Day began and ended: first and last Activity, as local minutes.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;
use serde::Serialize;
use utoipa::ToSchema;

use super::ActivityRow;
use crate::analysis::{Period, split_by_day};

/// The first and last Activity of one Day.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct DayBound {
    /// Local date.
    #[schema(value_type = String, format = Date)]
    pub date: NaiveDate,
    /// Minutes after local midnight when the first Activity began.
    pub first_minute: u32,
    /// Minutes after local midnight when the last Activity ended (at most 1440).
    pub last_minute: u32,
}

/// Minutes between the local midnight of `date` and `instant`, rounded down or up.
fn minutes_into(tz: Tz, date: NaiveDate, instant: DateTime<Utc>, round_up: bool) -> u32 {
    let local = instant.with_timezone(&tz).naive_local();
    let millis = (local - date.and_time(NaiveTime::MIN))
        .num_milliseconds()
        .max(0);
    let minutes = if round_up {
        (millis + 59_999) / 60_000
    } else {
        millis / 60_000
    };
    u32::try_from(minutes.min(24 * 60)).unwrap_or(24 * 60)
}

/// One entry per Day that has Activity, earliest first.
#[must_use]
pub fn day_bounds(period: &Period, rows: &[ActivityRow]) -> Vec<DayBound> {
    let mut found: BTreeMap<NaiveDate, (DateTime<Utc>, DateTime<Utc>)> = BTreeMap::new();
    for row in rows {
        for portion in split_by_day(period, row.start, row.end) {
            let entry = found
                .entry(portion.date)
                .or_insert((portion.start, portion.end));
            entry.0 = entry.0.min(portion.start);
            entry.1 = entry.1.max(portion.end);
        }
    }
    found
        .into_iter()
        .map(|(date, (first, last))| DayBound {
            date,
            first_minute: minutes_into(period.tz(), date, first, false),
            last_minute: minutes_into(period.tz(), date, last, true),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::row::at;
    use super::*;

    fn bounds(tz: &str, from: &str, to: &str, spans: &[(&str, &str)]) -> Vec<(String, u32, u32)> {
        let period = Period::parse(from, to, tz).unwrap();
        let rows: Vec<ActivityRow> = spans
            .iter()
            .map(|(s, e)| ActivityRow::new("App", "", at(s), at(e)))
            .collect();
        day_bounds(&period, &rows)
            .into_iter()
            .map(|b| (b.date.to_string(), b.first_minute, b.last_minute))
            .collect()
    }

    #[test]
    fn a_day_has_its_earliest_start_and_latest_end() {
        let found = bounds(
            "UTC",
            "2026-05-01",
            "2026-05-01",
            &[
                ("2026-05-01T14:00:00Z", "2026-05-01T15:30:00Z"),
                ("2026-05-01T08:15:00Z", "2026-05-01T09:00:00Z"),
            ],
        );
        assert_eq!(
            found,
            [("2026-05-01".to_owned(), 8 * 60 + 15, 15 * 60 + 30)]
        );
    }

    #[test]
    fn days_without_activity_are_left_out() {
        let found = bounds(
            "UTC",
            "2026-05-01",
            "2026-05-03",
            &[("2026-05-02T10:00:00Z", "2026-05-02T11:00:00Z")],
        );
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "2026-05-02");
    }

    #[test]
    fn a_night_session_ends_one_day_at_midnight_and_starts_the_next_at_zero() {
        let found = bounds(
            "UTC",
            "2026-05-01",
            "2026-05-02",
            &[("2026-05-01T23:00:00Z", "2026-05-02T01:30:00Z")],
        );
        assert_eq!(
            found,
            [
                ("2026-05-01".to_owned(), 23 * 60, 24 * 60),
                ("2026-05-02".to_owned(), 0, 90)
            ]
        );
    }

    #[test]
    fn times_are_local_to_the_zone() {
        let found = bounds(
            "Asia/Kolkata",
            "2026-05-01",
            "2026-05-01",
            &[("2026-05-01T04:00:00Z", "2026-05-01T05:00:00Z")],
        );
        assert_eq!(
            found,
            [("2026-05-01".to_owned(), 9 * 60 + 30, 10 * 60 + 30)]
        );
    }

    #[test]
    fn partial_minutes_widen_the_bounds() {
        let found = bounds(
            "UTC",
            "2026-05-01",
            "2026-05-01",
            &[("2026-05-01T08:00:40Z", "2026-05-01T09:00:20Z")],
        );
        assert_eq!(found, [("2026-05-01".to_owned(), 8 * 60, 9 * 60 + 1)]);
    }

    #[test]
    fn an_empty_period_has_no_bounds() {
        assert!(bounds("UTC", "2026-05-01", "2026-05-02", &[]).is_empty());
    }
}
