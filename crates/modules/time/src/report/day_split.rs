//! Splitting a Time Entry across the Days it touches.

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

use super::period::local_midnight;

/// The part of one entry that falls on one Day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayPart {
    /// The local Day.
    pub date: NaiveDate,
    /// Seconds of the entry inside that Day (and inside the Period).
    pub seconds: i64,
}

/// Cuts `[start, stop)` to `range`, then at each local midnight. Day lengths follow the
/// time zone, so a DST Day holds 23 or 25 hours.
#[must_use]
pub fn split(
    start: DateTime<Utc>,
    stop: DateTime<Utc>,
    range: (DateTime<Utc>, DateTime<Utc>),
    tz: Tz,
) -> Vec<DayPart> {
    let mut at = start.max(range.0);
    let end = stop.min(range.1);
    let mut parts = Vec::new();
    while at < end {
        let date = at.with_timezone(&tz).date_naive();
        let day_end = local_midnight(tz, date.succ_opt().unwrap_or(date));
        // Guards a zone whose midnight lookup would not move forward.
        let part_end = if day_end > at { day_end.min(end) } else { end };
        parts.push(DayPart {
            date,
            seconds: (part_end - at).num_seconds(),
        });
        at = part_end;
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }
    const WIDE: (&str, &str) = ("2026-01-01T00:00:00Z", "2027-01-01T00:00:00Z");
    fn wide() -> (DateTime<Utc>, DateTime<Utc>) {
        (t(WIDE.0), t(WIDE.1))
    }

    #[test]
    fn an_entry_inside_one_day_is_one_part() {
        let parts = split(
            t("2026-10-17T09:00:00Z"),
            t("2026-10-17T10:30:00Z"),
            wide(),
            chrono_tz::UTC,
        );
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].seconds, 5400);
    }

    #[test]
    fn a_midnight_crossing_entry_gets_a_part_on_each_day() {
        let parts = split(
            t("2026-10-17T23:00:00Z"),
            t("2026-10-18T01:30:00Z"),
            wide(),
            chrono_tz::UTC,
        );
        let seconds: Vec<i64> = parts.iter().map(|p| p.seconds).collect();
        assert_eq!(seconds, [3600, 5400]);
        assert_ne!(parts[0].date, parts[1].date);
    }

    #[test]
    fn midnight_is_local_to_the_time_zone() {
        let tz: Tz = "Asia/Tokyo".parse().unwrap();
        // 14:00Z-16:00Z is 23:00-01:00 in Tokyo.
        let parts = split(
            t("2026-10-17T14:00:00Z"),
            t("2026-10-17T16:00:00Z"),
            wide(),
            tz,
        );
        assert_eq!(
            parts.iter().map(|p| p.seconds).collect::<Vec<_>>(),
            [3600, 3600]
        );
    }

    #[test]
    fn parts_outside_the_period_are_clipped() {
        let range = (t("2026-10-18T00:00:00Z"), t("2026-10-19T00:00:00Z"));
        let parts = split(
            t("2026-10-17T23:00:00Z"),
            t("2026-10-18T01:00:00Z"),
            range,
            chrono_tz::UTC,
        );
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].seconds, 3600);
        assert!(
            split(
                t("2026-10-20T00:00:00Z"),
                t("2026-10-20T01:00:00Z"),
                range,
                chrono_tz::UTC
            )
            .is_empty()
        );
    }

    #[test]
    fn a_day_with_a_dst_change_is_split_at_its_real_midnights() {
        let tz: Tz = "Europe/Berlin".parse().unwrap();
        // A whole 25-hour Day (2026-10-25) plus an hour either side.
        let parts = split(
            t("2026-10-24T21:00:00Z"),
            t("2026-10-25T23:00:00Z"),
            wide(),
            tz,
        );
        let hours: Vec<i64> = parts.iter().map(|p| p.seconds / 3600).collect();
        assert_eq!(hours, [1, 25]);
    }

    fn hours(parts: &[DayPart]) -> Vec<i64> {
        parts.iter().map(|p| p.seconds / 3600).collect()
    }
    fn berlin() -> Tz {
        "Europe/Berlin".parse().unwrap()
    }

    #[test]
    fn a_zero_length_entry_has_no_parts() {
        let at = t("2026-10-17T09:00:00Z");
        assert!(split(at, at, wide(), chrono_tz::UTC).is_empty());
    }

    #[test]
    fn a_reversed_entry_has_no_parts() {
        let parts = split(
            t("2026-10-17T10:00:00Z"),
            t("2026-10-17T09:00:00Z"),
            wide(),
            chrono_tz::UTC,
        );
        assert!(parts.is_empty());
    }

    #[test]
    fn an_entry_stopping_exactly_at_midnight_has_no_zero_second_part_on_the_next_day() {
        let parts = split(
            t("2026-10-17T22:00:00Z"),
            t("2026-10-18T00:00:00Z"),
            wide(),
            chrono_tz::UTC,
        );
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].seconds, 7200);
        assert_eq!(parts[0].date.to_string(), "2026-10-17");
    }

    #[test]
    fn an_entry_starting_exactly_at_midnight_belongs_wholly_to_that_day() {
        let parts = split(
            t("2026-10-18T00:00:00Z"),
            t("2026-10-18T02:00:00Z"),
            wide(),
            chrono_tz::UTC,
        );
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].date.to_string(), "2026-10-18");
    }

    #[test]
    fn a_spring_forward_day_holds_23_hours() {
        // Berlin, 2026-03-29: 01:00 local on the 28th (00:00Z) to 01:00 local on the 30th.
        let parts = split(
            t("2026-03-28T22:00:00Z"),
            t("2026-03-29T23:00:00Z"),
            wide(),
            berlin(),
        );
        assert_eq!(hours(&parts), [1, 23, 1]);
    }

    #[test]
    fn an_entry_spanning_three_or_more_days_gets_a_part_on_each() {
        let parts = split(
            t("2026-10-16T12:00:00Z"),
            t("2026-10-19T06:00:00Z"),
            wide(),
            chrono_tz::UTC,
        );
        assert_eq!(hours(&parts), [12, 24, 24, 6]);
        let dates: Vec<String> = parts.iter().map(|p| p.date.to_string()).collect();
        assert_eq!(
            dates,
            ["2026-10-16", "2026-10-17", "2026-10-18", "2026-10-19"]
        );
    }

    #[test]
    fn parts_after_the_period_end_are_clipped() {
        let range = (t("2026-10-17T00:00:00Z"), t("2026-10-18T00:00:00Z"));
        let parts = split(
            t("2026-10-17T23:00:00Z"),
            t("2026-10-18T05:00:00Z"),
            range,
            chrono_tz::UTC,
        );
        assert_eq!(hours(&parts), [1]);
    }

    #[test]
    fn an_entry_ending_exactly_at_the_period_start_has_no_part() {
        let range = (t("2026-10-18T00:00:00Z"), t("2026-10-19T00:00:00Z"));
        let parts = split(
            t("2026-10-17T22:00:00Z"),
            t("2026-10-18T00:00:00Z"),
            range,
            chrono_tz::UTC,
        );
        assert!(parts.is_empty());
    }
}
