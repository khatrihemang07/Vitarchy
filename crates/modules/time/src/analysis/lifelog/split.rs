//! How each Day divides: hours per project, and the part nobody logged.
//!
//! Project time is each entry's own time, so entries that overlap each count in
//! full (a Day's projects can add up to more than the Day). Unlogged time is the
//! Day minus the *union* of its entries, so overlap is never counted twice there.
//! The part of today that has not happened yet is neither logged nor unlogged.

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::LifeEntry;
use super::entry::{millis, seconds};
use crate::analysis::{Period, split_by_day};

/// Seconds one project took on one Day.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProjectSeconds {
    /// The project's name.
    pub project: String,
    /// Seconds, counting each entry's own time.
    pub seconds: i64,
}

/// One Day's division.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct DaySplit {
    /// Local date.
    #[schema(value_type = String, format = Date)]
    pub date: NaiveDate,
    /// How much of the Day has happened: its length, less any part still in the future.
    pub elapsed_seconds: i64,
    /// Elapsed time covered by no entry.
    pub unlogged_seconds: i64,
    /// Time per project, longest first.
    pub by_project: Vec<ProjectSeconds>,
}

#[derive(Default)]
struct DayWork {
    spans: Vec<(DateTime<Utc>, DateTime<Utc>)>,
    per_project: HashMap<String, i64>,
}

/// One [`DaySplit`] for every Day of the Period, earliest first; `now` ends the present Day early.
#[must_use]
pub fn day_splits(period: &Period, entries: &[LifeEntry], now: DateTime<Utc>) -> Vec<DaySplit> {
    let mut work: BTreeMap<NaiveDate, DayWork> = BTreeMap::new();
    for entry in entries {
        for portion in split_by_day(period, entry.start, entry.end) {
            let day = work.entry(portion.date).or_default();
            day.spans.push((portion.start, portion.end));
            *day.per_project.entry(entry.project.clone()).or_default() += millis(&portion);
        }
    }
    period
        .days()
        .iter()
        .map(|day| {
            let mut done = work.remove(&day.date).unwrap_or_default();
            let until = day.end.min(now);
            let elapsed = (until - day.start).num_milliseconds().max(0);
            let covered = union_millis(&mut done.spans, day.start, until);
            let mut by_project: Vec<ProjectSeconds> = done
                .per_project
                .into_iter()
                .map(|(project, ms)| ProjectSeconds {
                    project,
                    seconds: seconds(ms),
                })
                .collect();
            by_project.sort_by(|a, b| {
                b.seconds
                    .cmp(&a.seconds)
                    .then_with(|| a.project.cmp(&b.project))
            });
            DaySplit {
                date: day.date,
                elapsed_seconds: seconds(elapsed),
                unlogged_seconds: seconds(elapsed - covered),
                by_project,
            }
        })
        .collect()
}

/// Milliseconds covered by the union of `spans`, cut to `[from, until)`.
fn union_millis(
    spans: &mut [(DateTime<Utc>, DateTime<Utc>)],
    from: DateTime<Utc>,
    until: DateTime<Utc>,
) -> i64 {
    spans.sort();
    let mut total = 0;
    let mut reached = from;
    for &(start, end) in spans.iter() {
        let (start, end) = (start.max(reached), end.min(until));
        if end > start {
            total += (end - start).num_milliseconds();
            reached = end;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::super::entry::entry;
    use super::*;

    const HOUR: i64 = 3600;
    const FAR_FUTURE: &str = "2099-01-01T00:00:00Z";

    fn splits(
        tz: &str,
        from: &str,
        to: &str,
        rows: &[(&str, &str, &str)],
        now: &str,
    ) -> Vec<DaySplit> {
        let period = Period::parse(from, to, tz).unwrap();
        let entries: Vec<LifeEntry> = rows.iter().map(|(p, s, e)| entry(p, "x", s, e)).collect();
        day_splits(&period, &entries, now.parse().unwrap())
    }

    fn project(split: &DaySplit, name: &str) -> i64 {
        split
            .by_project
            .iter()
            .find(|p| p.project == name)
            .map_or(0, |p| p.seconds)
    }

    #[test]
    fn a_day_with_one_entry_has_the_rest_unlogged() {
        let out = splits(
            "UTC",
            "2026-05-01",
            "2026-05-01",
            &[("office", "2026-05-01T09:00:00Z", "2026-05-01T17:00:00Z")],
            FAR_FUTURE,
        );
        assert_eq!(project(&out[0], "office"), 8 * HOUR);
        assert_eq!(
            (out[0].elapsed_seconds, out[0].unlogged_seconds),
            (24 * HOUR, 16 * HOUR)
        );
    }

    #[test]
    fn sleep_over_midnight_is_split_between_the_two_days() {
        let out = splits(
            "UTC",
            "2026-05-01",
            "2026-05-02",
            &[("sleep", "2026-05-01T23:00:00Z", "2026-05-02T07:00:00Z")],
            FAR_FUTURE,
        );
        assert_eq!(
            (project(&out[0], "sleep"), project(&out[1], "sleep")),
            (HOUR, 7 * HOUR)
        );
        assert_eq!(
            (out[0].unlogged_seconds, out[1].unlogged_seconds),
            (23 * HOUR, 17 * HOUR)
        );
    }

    #[test]
    fn overlapping_entries_count_in_projects_but_not_twice_in_unlogged() {
        let out = splits(
            "UTC",
            "2026-05-01",
            "2026-05-01",
            &[
                ("office", "2026-05-01T09:00:00Z", "2026-05-01T12:00:00Z"),
                ("body", "2026-05-01T11:00:00Z", "2026-05-01T13:00:00Z"),
            ],
            FAR_FUTURE,
        );
        assert_eq!(
            (project(&out[0], "office"), project(&out[0], "body")),
            (3 * HOUR, 2 * HOUR)
        );
        // The union is 09:00-13:00, four hours.
        assert_eq!(out[0].unlogged_seconds, 20 * HOUR);
    }

    #[test]
    fn an_entry_inside_another_does_not_shrink_unlogged_further() {
        let out = splits(
            "UTC",
            "2026-05-01",
            "2026-05-01",
            &[
                ("office", "2026-05-01T09:00:00Z", "2026-05-01T17:00:00Z"),
                ("body", "2026-05-01T10:00:00Z", "2026-05-01T11:00:00Z"),
            ],
            FAR_FUTURE,
        );
        assert_eq!(out[0].unlogged_seconds, 16 * HOUR);
    }

    #[test]
    fn the_future_part_of_today_is_not_unlogged() {
        let out = splits(
            "UTC",
            "2026-05-01",
            "2026-05-01",
            &[("office", "2026-05-01T09:00:00Z", "2026-05-01T10:00:00Z")],
            "2026-05-01T12:00:00Z",
        );
        assert_eq!(
            (out[0].elapsed_seconds, out[0].unlogged_seconds),
            (12 * HOUR, 11 * HOUR)
        );
    }

    #[test]
    fn days_that_have_not_begun_have_nothing_unlogged() {
        let out = splits(
            "UTC",
            "2026-05-01",
            "2026-05-03",
            &[],
            "2026-05-02T06:00:00Z",
        );
        let unlogged: Vec<i64> = out.iter().map(|d| d.unlogged_seconds).collect();
        assert_eq!(unlogged, [24 * HOUR, 6 * HOUR, 0]);
        assert_eq!(out[2].elapsed_seconds, 0);
    }

    #[test]
    fn an_entry_still_in_the_future_of_today_does_not_make_unlogged_negative() {
        let out = splits(
            "UTC",
            "2026-05-01",
            "2026-05-01",
            &[("office", "2026-05-01T15:00:00Z", "2026-05-01T17:00:00Z")],
            "2026-05-01T12:00:00Z",
        );
        assert_eq!(out[0].unlogged_seconds, 12 * HOUR);
    }

    #[test]
    fn a_dst_day_is_as_long_as_it_really_was() {
        let out = splits(
            "America/New_York",
            "2026-03-08",
            "2026-03-08",
            &[],
            FAR_FUTURE,
        );
        assert_eq!(out[0].elapsed_seconds, 23 * HOUR);
        assert_eq!(out[0].unlogged_seconds, 23 * HOUR);
    }

    #[test]
    fn local_midnight_decides_the_day() {
        // 20:00Z to 22:00Z is 01:30-03:30 the next day in Kolkata.
        let out = splits(
            "Asia/Kolkata",
            "2026-05-01",
            "2026-05-02",
            &[("sleep", "2026-05-01T20:00:00Z", "2026-05-01T22:00:00Z")],
            FAR_FUTURE,
        );
        assert_eq!(
            (project(&out[0], "sleep"), project(&out[1], "sleep")),
            (0, 2 * HOUR)
        );
    }

    #[test]
    fn projects_come_longest_first_and_every_day_is_listed() {
        let out = splits(
            "UTC",
            "2026-05-01",
            "2026-05-02",
            &[
                ("body", "2026-05-01T09:00:00Z", "2026-05-01T10:00:00Z"),
                ("office", "2026-05-01T10:00:00Z", "2026-05-01T14:00:00Z"),
            ],
            FAR_FUTURE,
        );
        assert_eq!(out.len(), 2);
        let names: Vec<&str> = out[0]
            .by_project
            .iter()
            .map(|p| p.project.as_str())
            .collect();
        assert_eq!(names, ["office", "body"]);
        assert!(out[1].by_project.is_empty());
    }
}
