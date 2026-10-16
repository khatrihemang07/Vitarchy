//! Habits: when a frequent description usually starts, ends, and how long it lasts.
//!
//! An occurrence is a whole entry (not its per-Day portions) that **starts** inside the
//! Period, so a night's sleep is one 23:30 to 07:00 occurrence of 7.5 h. Entries with no
//! description are not a habit and are left out. Clock times are local to the Period's zone.

use std::collections::{BTreeSet, HashMap};

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;
use serde::Serialize;
use utoipa::ToSchema;

use super::LifeEntry;
use super::spread::{ClockSpread, DurationSpread, clock_spread, duration_spread};
use crate::analysis::Period;

/// A description must occur on at least this many Days to get statistics.
pub const MIN_DAYS: usize = 5;

/// The usual start, end and length of one description.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct DescriptionStat {
    /// The description.
    pub description: String,
    /// Days on which an occurrence started.
    pub days: i64,
    /// Occurrences (a Day may have several).
    pub entries: i64,
    /// When it usually starts, local time.
    pub start: ClockSpread,
    /// When it usually ends, local time.
    pub end: ClockSpread,
    /// How long it usually lasts.
    pub duration: DurationSpread,
}

#[derive(Default)]
struct Occurrences {
    dates: BTreeSet<NaiveDate>,
    starts: Vec<f64>,
    ends: Vec<f64>,
    seconds: Vec<i64>,
}

/// Minutes after local midnight of `instant` in `tz`.
fn minute_of_day(tz: Tz, instant: DateTime<Utc>) -> f64 {
    let local = instant.with_timezone(&tz).naive_local();
    f64::from(chrono::Timelike::num_seconds_from_midnight(&local.time())) / 60.0
}

/// Statistics for each description seen on at least [`MIN_DAYS`] Days, most Days first.
#[must_use]
pub fn description_stats(period: &Period, entries: &[LifeEntry]) -> Vec<DescriptionStat> {
    let tz = period.tz();
    let mut seen: HashMap<&str, Occurrences> = HashMap::new();
    for entry in entries {
        if entry.description.is_empty()
            || entry.start < period.start()
            || entry.start >= period.end()
        {
            continue;
        }
        let found = seen.entry(&entry.description).or_default();
        found
            .dates
            .insert(entry.start.with_timezone(&tz).date_naive());
        found.starts.push(minute_of_day(tz, entry.start));
        found.ends.push(minute_of_day(tz, entry.end));
        found.seconds.push((entry.end - entry.start).num_seconds());
    }
    let mut stats: Vec<DescriptionStat> = seen
        .into_iter()
        .filter(|(_, found)| found.dates.len() >= MIN_DAYS)
        .filter_map(|(description, found)| {
            Some(DescriptionStat {
                description: description.to_owned(),
                days: i64::try_from(found.dates.len()).unwrap_or(i64::MAX),
                entries: i64::try_from(found.seconds.len()).unwrap_or(i64::MAX),
                start: clock_spread(&found.starts)?,
                end: clock_spread(&found.ends)?,
                duration: duration_spread(&found.seconds)?,
            })
        })
        .collect();
    stats.sort_by(|a, b| {
        b.days
            .cmp(&a.days)
            .then_with(|| a.description.cmp(&b.description))
    });
    stats
}

#[cfg(test)]
mod tests {
    use super::super::entry::entry;
    use super::*;

    /// One entry a day from `first` for `count` days, starting `start` and ending `end` (clock text, UTC).
    fn daily(
        description: &str,
        first: u32,
        count: u32,
        start: &str,
        end: &str,
        end_next_day: bool,
    ) -> Vec<LifeEntry> {
        (0..count)
            .map(|i| {
                let day = first + i;
                let end_day = if end_next_day { day + 1 } else { day };
                entry(
                    "p",
                    description,
                    &format!("2026-05-{day:02}T{start}:00Z"),
                    &format!("2026-05-{end_day:02}T{end}:00Z"),
                )
            })
            .collect()
    }

    fn stats(tz: &str, entries: &[LifeEntry]) -> Vec<DescriptionStat> {
        let period = Period::parse("2026-05-01", "2026-05-20", tz).unwrap();
        description_stats(&period, entries)
    }

    #[test]
    fn a_steady_habit_has_its_usual_start_end_and_length() {
        let found = stats("UTC", &daily("workout", 1, 6, "18:30", "19:10", false));
        assert_eq!(found.len(), 1);
        let s = &found[0];
        assert_eq!(
            (s.description.as_str(), s.days, s.entries),
            ("workout", 6, 6)
        );
        assert_eq!(s.start.median_minute, 18 * 60 + 30);
        assert_eq!(s.end.median_minute, 19 * 60 + 10);
        assert_eq!(s.duration.median_seconds, 40 * 60);
    }

    #[test]
    fn a_description_on_fewer_than_five_days_is_left_out() {
        assert!(stats("UTC", &daily("workout", 1, 4, "18:30", "19:10", false)).is_empty());
        assert_eq!(
            stats("UTC", &daily("workout", 1, 5, "18:30", "19:10", false)).len(),
            1
        );
    }

    #[test]
    fn several_entries_on_one_day_count_as_one_day() {
        let mut rows = daily("bath", 1, 4, "07:00", "07:15", false);
        rows.push(entry(
            "p",
            "bath",
            "2026-05-04T21:00:00Z",
            "2026-05-04T21:15:00Z",
        ));
        assert!(stats("UTC", &rows).is_empty());
    }

    #[test]
    fn sleep_that_starts_around_midnight_has_a_usual_start_around_midnight() {
        let mut rows = Vec::new();
        for (i, start) in ["23:30", "23:50", "00:10", "00:30", "01:00", "23:40"]
            .iter()
            .enumerate()
        {
            let day = 2 + u32::try_from(i).unwrap();
            // Starts after midnight belong to the next date in the same night.
            let begin = format!("2026-05-{day:02}T{start}:00Z");
            let end = format!("2026-05-{day:02}T07:30:00Z");
            let (begin, end) = if start.starts_with("23") {
                (begin, format!("2026-05-{:02}T07:30:00Z", day + 1))
            } else {
                (begin, end)
            };
            rows.push(entry("p", "sleep", &begin, &end));
        }
        let s = &stats("UTC", &rows)[0];
        assert!(
            s.start.median_minute >= 23 * 60 + 40 || s.start.median_minute <= 20,
            "{}",
            s.start.median_minute
        );
        assert!(s.start.p25_minute >= 23 * 60, "p25 {}", s.start.p25_minute);
        assert!(s.start.p75_minute <= 60, "p75 {}", s.start.p75_minute);
        assert_eq!(s.end.median_minute, 7 * 60 + 30);
    }

    #[test]
    fn a_night_across_midnight_lasts_its_whole_length() {
        let s = &stats("UTC", &daily("sleep", 1, 5, "23:00", "07:00", true))[0];
        assert_eq!(s.duration.median_seconds, 8 * 3600);
    }

    #[test]
    fn clock_times_are_local_to_the_period_zone() {
        // 04:00Z is 09:30 in Kolkata.
        let s = &stats("Asia/Kolkata", &daily("job", 2, 5, "04:00", "08:00", false))[0];
        assert_eq!(s.start.median_minute, 9 * 60 + 30);
    }

    #[test]
    fn entries_starting_before_the_period_or_without_a_description_are_not_occurrences() {
        let mut rows = daily("job", 1, 5, "09:00", "10:00", false);
        rows.push(entry(
            "p",
            "job",
            "2026-04-30T09:00:00Z",
            "2026-04-30T10:00:00Z",
        ));
        rows.extend(daily("", 1, 8, "12:00", "13:00", false));
        let found = stats("UTC", &rows);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].entries, 5);
    }

    #[test]
    fn the_most_frequent_description_comes_first() {
        let mut rows = daily("b_less", 1, 5, "09:00", "10:00", false);
        rows.extend(daily("a_more", 1, 7, "11:00", "12:00", false));
        let names: Vec<String> = stats("UTC", &rows)
            .into_iter()
            .map(|s| s.description)
            .collect();
        assert_eq!(names, ["a_more", "b_less"]);
    }
}
