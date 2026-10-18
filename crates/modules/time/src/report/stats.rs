//! Numbers for one group: Days, entries, total and the spread per Day and per entry.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;

/// One entry's time on one Day, as it counts toward a group.
#[derive(Debug, Clone, Copy)]
pub struct Part {
    /// The entry's Toggl id.
    pub entry: i64,
    /// The Day.
    pub date: NaiveDate,
    /// Seconds on that Day, inside the Period.
    pub seconds: i64,
}

/// Average, shortest and longest, in seconds.
// The `_s` suffix is the API's unit convention (`total_s`), not redundancy.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, ToSchema)]
pub struct Spread {
    /// Mean, rounded to the nearest second.
    pub avg_s: i64,
    /// Shortest.
    pub min_s: i64,
    /// Longest.
    pub max_s: i64,
}

/// What a group amounts to over the Period.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, ToSchema)]
pub struct Stats {
    /// Days with time in this group.
    pub days: i64,
    /// Distinct Time Entries in this group.
    pub entries: i64,
    /// Seconds in total.
    pub total_s: i64,
    /// Spread of the group's daily totals, over the Days it appears on only.
    pub per_day: Spread,
    /// Spread of each entry's time inside the Period.
    pub per_entry: Spread,
}

/// Average (rounded), shortest and longest of `values`; all 0 with none.
pub fn spread(values: impl ExactSizeIterator<Item = i64> + Clone) -> Spread {
    let count = i64::try_from(values.len()).unwrap_or(i64::MAX);
    if count == 0 {
        return Spread::default();
    }
    let sum: i64 = values.clone().sum();
    Spread {
        avg_s: (sum + count / 2) / count,
        min_s: values.clone().min().unwrap_or(0),
        max_s: values.max().unwrap_or(0),
    }
}

/// Stats for the parts of one group. Parts without time are ignored.
#[must_use]
pub fn compute(parts: &[Part]) -> Stats {
    let mut by_day: BTreeMap<NaiveDate, i64> = BTreeMap::new();
    let mut by_entry: BTreeMap<i64, i64> = BTreeMap::new();
    for part in parts.iter().filter(|p| p.seconds > 0) {
        *by_day.entry(part.date).or_default() += part.seconds;
        *by_entry.entry(part.entry).or_default() += part.seconds;
    }
    let count = |n: usize| i64::try_from(n).unwrap_or(i64::MAX);
    Stats {
        days: count(by_day.len()),
        entries: count(by_entry.len()),
        total_s: by_day.values().sum(),
        per_day: spread(by_day.values().copied()),
        per_entry: spread(by_entry.values().copied()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, day).unwrap()
    }
    fn part(entry: i64, day: u32, seconds: i64) -> Part {
        Part {
            entry,
            date: d(day),
            seconds,
        }
    }

    #[test]
    fn two_entries_on_one_day_are_one_day_but_two_entries() {
        let stats = compute(&[part(1, 1, 3 * 3600), part(2, 1, 4 * 3600)]);
        assert_eq!((stats.days, stats.entries, stats.total_s), (1, 2, 7 * 3600));
        assert_eq!(
            stats.per_day,
            Spread {
                avg_s: 7 * 3600,
                min_s: 7 * 3600,
                max_s: 7 * 3600
            }
        );
        assert_eq!(
            stats.per_entry,
            Spread {
                avg_s: 12600,
                min_s: 3 * 3600,
                max_s: 4 * 3600
            }
        );
    }

    #[test]
    fn the_day_average_only_counts_days_the_group_appears_on() {
        let stats = compute(&[part(1, 1, 3600), part(2, 5, 7200)]);
        assert_eq!(stats.days, 2);
        assert_eq!(stats.per_day.avg_s, 5400);
    }

    #[test]
    fn an_entry_spanning_midnight_is_one_entry_on_two_days() {
        let stats = compute(&[part(1, 1, 3600), part(1, 2, 1800)]);
        assert_eq!((stats.days, stats.entries), (2, 1));
        assert_eq!(stats.per_entry.max_s, 5400);
    }

    #[test]
    fn averages_round_to_the_nearest_second() {
        let stats = compute(&[part(1, 1, 1), part(2, 2, 1), part(3, 3, 2)]);
        assert_eq!(stats.per_day.avg_s, 1); // 4/3 = 1.33
        let stats = compute(&[part(1, 1, 1), part(2, 2, 2)]);
        assert_eq!(stats.per_day.avg_s, 2); // 1.5 rounds up
    }

    #[test]
    fn no_parts_means_all_zero() {
        assert_eq!(compute(&[]), Stats::default());
        assert_eq!(compute(&[part(1, 1, 0)]), Stats::default());
    }
}
