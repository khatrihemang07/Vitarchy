//! Splits a `[start, end)` span into the part that falls inside each local Day
//! of a [`Period`], so midnight-crossing rows count toward each Day they touch.

use chrono::{DateTime, NaiveDate, Utc};

use super::period::Period;

/// The part of a span inside one Day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayPortion {
    /// The local date of the Day.
    pub date: NaiveDate,
    /// Where the portion begins (inclusive).
    pub start: DateTime<Utc>,
    /// Where the portion ends (exclusive).
    pub end: DateTime<Utc>,
}

impl DayPortion {
    /// Length in seconds.
    #[must_use]
    pub fn seconds(&self) -> i64 {
        (self.end - self.start).num_seconds()
    }
}

/// The portions of `[start, end)` inside the Period's Days, earliest first.
/// Parts outside the Period, and empty or reversed spans, yield nothing.
#[must_use]
pub fn split_by_day(period: &Period, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<DayPortion> {
    if end <= start {
        return Vec::new();
    }
    let days = period.days();
    // Days are sorted and contiguous, so skip straight to the first that ends after `start`.
    let first = days.partition_point(|day| day.end <= start);
    days[first..]
        .iter()
        .take_while(|day| day.start < end)
        .map(|day| DayPortion {
            date: day.date,
            start: start.max(day.start),
            end: end.min(day.end),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc(text: &str) -> DateTime<Utc> {
        text.parse().unwrap()
    }

    fn portions(period: &Period, start: &str, end: &str) -> Vec<(String, i64)> {
        split_by_day(period, utc(start), utc(end))
            .iter()
            .map(|p| (p.date.to_string(), p.seconds()))
            .collect()
    }

    const HOUR: i64 = 3600;

    #[test]
    fn a_span_inside_one_day_is_one_portion() {
        let period = Period::parse("2026-05-01", "2026-05-03", "UTC").unwrap();
        assert_eq!(
            portions(&period, "2026-05-02T10:00:00Z", "2026-05-02T12:00:00Z"),
            [("2026-05-02".to_owned(), 2 * HOUR)]
        );
    }

    #[test]
    fn a_span_over_midnight_is_split_in_two() {
        let period = Period::parse("2026-05-01", "2026-05-03", "UTC").unwrap();
        assert_eq!(
            portions(&period, "2026-05-01T22:00:00Z", "2026-05-02T06:00:00Z"),
            [
                ("2026-05-01".to_owned(), 2 * HOUR),
                ("2026-05-02".to_owned(), 6 * HOUR)
            ]
        );
    }

    #[test]
    fn a_111_hour_span_covers_every_day_it_touches_and_sums_back() {
        let period = Period::parse("2026-05-01", "2026-05-10", "UTC").unwrap();
        let split = portions(&period, "2026-05-02T20:00:00Z", "2026-05-07T11:00:00Z");
        let dates: Vec<&str> = split.iter().map(|(d, _)| d.as_str()).collect();
        assert_eq!(
            dates,
            [
                "2026-05-02",
                "2026-05-03",
                "2026-05-04",
                "2026-05-05",
                "2026-05-06",
                "2026-05-07"
            ]
        );
        assert_eq!(split.iter().map(|(_, s)| s).sum::<i64>(), 111 * HOUR);
        assert_eq!(split[1].1, 24 * HOUR);
    }

    #[test]
    fn only_the_part_inside_the_period_counts() {
        let period = Period::parse("2026-05-02", "2026-05-02", "UTC").unwrap();
        assert_eq!(
            portions(&period, "2026-05-01T20:00:00Z", "2026-05-03T04:00:00Z"),
            [("2026-05-02".to_owned(), 24 * HOUR)]
        );
        assert!(portions(&period, "2026-04-20T00:00:00Z", "2026-04-21T00:00:00Z").is_empty());
        assert!(portions(&period, "2026-06-01T00:00:00Z", "2026-06-02T00:00:00Z").is_empty());
    }

    #[test]
    fn zero_length_and_reversed_spans_yield_nothing() {
        let period = Period::parse("2026-05-01", "2026-05-03", "UTC").unwrap();
        assert!(portions(&period, "2026-05-02T10:00:00Z", "2026-05-02T10:00:00Z").is_empty());
        assert!(portions(&period, "2026-05-02T10:00:00Z", "2026-05-02T09:00:00Z").is_empty());
    }

    #[test]
    fn a_span_ending_exactly_at_midnight_stays_in_its_day() {
        let period = Period::parse("2026-05-01", "2026-05-02", "UTC").unwrap();
        assert_eq!(
            portions(&period, "2026-05-01T23:00:00Z", "2026-05-02T00:00:00Z"),
            [("2026-05-01".to_owned(), HOUR)]
        );
    }

    #[test]
    fn local_midnight_decides_the_split_not_utc_midnight() {
        // 20:00Z is 01:30 the next day in Kolkata.
        let period = Period::parse("2026-01-09", "2026-01-11", "Asia/Kolkata").unwrap();
        assert_eq!(
            portions(&period, "2026-01-09T17:30:00Z", "2026-01-09T19:30:00Z"),
            [
                ("2026-01-09".to_owned(), HOUR),
                ("2026-01-10".to_owned(), HOUR)
            ]
        );
    }

    #[test]
    fn a_night_across_spring_forward_loses_the_skipped_hour() {
        // New York 2026-03-07 22:00 EST to 2026-03-08 08:00 EDT: 9 real hours.
        let period = Period::parse("2026-03-07", "2026-03-08", "America/New_York").unwrap();
        let split = portions(&period, "2026-03-08T03:00:00Z", "2026-03-08T12:00:00Z");
        assert_eq!(
            split,
            [
                ("2026-03-07".to_owned(), 2 * HOUR),
                ("2026-03-08".to_owned(), 7 * HOUR)
            ]
        );
        assert_eq!(split.iter().map(|(_, s)| s).sum::<i64>(), 9 * HOUR);
    }

    #[test]
    fn a_night_across_fall_back_gains_the_repeated_hour() {
        // New York 2026-10-31 22:00 EDT to 2026-11-01 08:00 EST: 11 real hours.
        let period = Period::parse("2026-10-31", "2026-11-01", "America/New_York").unwrap();
        let split = portions(&period, "2026-11-01T02:00:00Z", "2026-11-01T13:00:00Z");
        assert_eq!(
            split,
            [
                ("2026-10-31".to_owned(), 2 * HOUR),
                ("2026-11-01".to_owned(), 9 * HOUR)
            ]
        );
    }

    #[test]
    fn a_full_dst_day_sums_to_its_real_length() {
        let period = Period::parse("2026-03-08", "2026-03-08", "America/New_York").unwrap();
        let split = portions(&period, "2026-03-07T00:00:00Z", "2026-03-10T00:00:00Z");
        assert_eq!(split, [("2026-03-08".to_owned(), 23 * HOUR)]);
    }
}
