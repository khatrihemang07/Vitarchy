//! Finds Routines the person has not saved yet: sequences of descriptions that recur in order.
//!
//! Within one Day, entries are taken in start order (consecutive entries with the same
//! description count as one). A sequence links descriptions that follow each other with at most
//! [`MAX_BETWEEN`] unrelated entries in between and no more than [`MAX_GAP`] of time between one
//! ending and the next starting. Sequences of 2 to [`MAX_STEPS`] distinct descriptions seen on
//! at least [`MIN_DAYS`] Days are ranked by Days covered times length, and a sequence that is
//! part of a better one (or of a saved Routine) is dropped.

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::analysis::Period;
use crate::analysis::lifelog::{LifeEntry, MIN_DAYS};

/// The most Steps a suggested Routine has.
pub const MAX_STEPS: usize = 5;
/// The most suggestions offered at once: past these the list is noise.
pub const MAX_SUGGESTIONS: usize = 6;
/// How many unrelated entries may sit between two linked ones.
const MAX_BETWEEN: usize = 1;
/// The longest wait between one entry ending and the next starting.
const MAX_GAP: Duration = Duration::hours(3);

/// A recurring sequence of descriptions, offered to become a Routine.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct Suggestion {
    /// The descriptions in order.
    pub steps: Vec<String>,
    /// Days on which the whole sequence happened.
    pub days: i64,
}

/// Consecutive entries with one description, merged.
struct Run<'a> {
    description: &'a str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

/// Suggestions for the Period, best first, leaving out what `saved` (description sequences of
/// existing Routines) already covers.
#[must_use]
pub fn suggest(period: &Period, entries: &[LifeEntry], saved: &[Vec<String>]) -> Vec<Suggestion> {
    let mut counts: HashMap<Vec<&str>, usize> = HashMap::new();
    for runs in runs_by_day(period, entries).values() {
        for chain in day_chains(runs) {
            *counts.entry(chain).or_default() += 1;
        }
    }
    let saved: Vec<Vec<String>> = saved
        .iter()
        .map(|steps| steps.iter().map(|s| s.trim().to_lowercase()).collect())
        .collect();
    let mut ranked: Vec<(Vec<&str>, usize)> = counts
        .into_iter()
        .filter(|(_, days)| *days >= MIN_DAYS)
        .collect();
    ranked.sort_by(|(a, a_days), (b, b_days)| {
        (b_days * b.len())
            .cmp(&(a_days * a.len()))
            .then_with(|| a.cmp(b))
    });
    let mut kept: Vec<(Vec<&str>, usize)> = vec![];
    for (chain, days) in ranked {
        let lowered: Vec<String> = chain.iter().map(|s| s.to_lowercase()).collect();
        let covered = saved.iter().any(|s| is_part_of(&lowered, s))
            || kept.iter().any(|(better, _)| is_part_of(&chain, better));
        if !covered {
            kept.push((chain, days));
        }
    }
    kept.into_iter()
        .take(MAX_SUGGESTIONS)
        .map(|(chain, days)| Suggestion {
            steps: chain.into_iter().map(str::to_owned).collect(),
            days: i64::try_from(days).unwrap_or(i64::MAX),
        })
        .collect()
}

/// Whether `part` appears in `whole` in the same order (not necessarily next to each other).
fn is_part_of<T: PartialEq>(part: &[T], whole: &[T]) -> bool {
    let mut rest = whole.iter();
    part.iter().all(|p| rest.any(|w| w == p))
}

/// Each Day's described entries in start order, merged into runs. An entry belongs to the Day it starts in.
fn runs_by_day<'a>(period: &Period, entries: &'a [LifeEntry]) -> BTreeMap<NaiveDate, Vec<Run<'a>>> {
    let mut days: BTreeMap<NaiveDate, Vec<&LifeEntry>> = BTreeMap::new();
    for entry in entries {
        if entry.description.is_empty()
            || entry.start < period.start()
            || entry.start >= period.end()
        {
            continue;
        }
        let date = entry.start.with_timezone(&period.tz()).date_naive();
        days.entry(date).or_default().push(entry);
    }
    days.into_iter()
        .map(|(date, mut day)| {
            day.sort_by_key(|e| e.start);
            let mut runs: Vec<Run> = vec![];
            for entry in day {
                match runs.last_mut() {
                    Some(run) if run.description == entry.description => {
                        run.end = run.end.max(entry.end);
                    }
                    _ => runs.push(Run {
                        description: &entry.description,
                        start: entry.start,
                        end: entry.end,
                    }),
                }
            }
            (date, runs)
        })
        .collect()
}

/// Every linked sequence of 2 to [`MAX_STEPS`] distinct descriptions in one Day's runs.
fn day_chains<'a>(runs: &[Run<'a>]) -> HashSet<Vec<&'a str>> {
    let mut found = HashSet::new();
    for first in 0..runs.len() {
        extend(runs, &[first], &mut found);
    }
    found
}

fn extend<'a>(runs: &[Run<'a>], chain: &[usize], found: &mut HashSet<Vec<&'a str>>) {
    let last = chain[chain.len() - 1];
    let reach = (last + 1 + MAX_BETWEEN).min(runs.len() - 1);
    for next in last + 1..=reach {
        let linked = runs[next].start - runs[last].end <= MAX_GAP
            && !chain
                .iter()
                .any(|i| runs[*i].description == runs[next].description);
        if !linked {
            continue;
        }
        let longer: Vec<usize> = chain.iter().copied().chain([next]).collect();
        found.insert(longer.iter().map(|i| runs[*i].description).collect());
        if longer.len() < MAX_STEPS {
            extend(runs, &longer, found);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::entry;
    use super::*;

    fn period() -> Period {
        Period::parse("2026-05-01", "2026-05-31", "UTC").unwrap()
    }

    fn at(day: u32, clock: &str) -> String {
        format!("2026-05-{day:02}T{clock}:00Z")
    }

    /// The commute-office-commute day: `(description, start, end)` per entry.
    fn day_of(day: u32, parts: &[(&str, &str, &str)]) -> Vec<LifeEntry> {
        parts
            .iter()
            .map(|(d, s, e)| entry("p", d, &at(day, s), &at(day, e)))
            .collect()
    }

    const OFFICE: [(&str, &str, &str); 3] = [
        ("travel to_office", "08:00", "09:00"),
        ("inside_office", "09:15", "17:00"),
        ("travel from_office", "17:10", "18:00"),
    ];

    fn office_days(count: u32) -> Vec<LifeEntry> {
        (1..=count).flat_map(|d| day_of(d, &OFFICE)).collect()
    }

    fn steps(s: &Suggestion) -> Vec<&str> {
        s.steps.iter().map(String::as_str).collect()
    }

    #[test]
    fn an_office_day_is_suggested_once_as_the_whole_chain() {
        let found = suggest(&period(), &office_days(8), &[]);
        assert_eq!(found.len(), 1);
        assert_eq!(
            steps(&found[0]),
            ["travel to_office", "inside_office", "travel from_office"]
        );
        assert_eq!(found[0].days, 8);
    }

    #[test]
    fn a_sequence_on_fewer_than_five_days_is_not_suggested() {
        assert!(suggest(&period(), &office_days(4), &[]).is_empty());
    }

    #[test]
    fn one_unrelated_entry_between_steps_is_allowed_but_two_are_not() {
        let with_noise = |noise: usize| -> Vec<LifeEntry> {
            (1..=6)
                .flat_map(|d| {
                    let mut parts = vec![("a", "08:00", "08:30")];
                    // Different noise each Day, so the noise itself never recurs.
                    let names = [format!("n1-{d}"), format!("n2-{d}")];
                    let times = [("08:40", "08:45"), ("08:50", "08:55")];
                    parts.extend((0..noise).map(|i| (names[i].as_str(), times[i].0, times[i].1)));
                    parts.push(("b", "09:00", "09:30"));
                    day_of(d, &parts)
                })
                .collect()
        };
        let one = suggest(&period(), &with_noise(1), &[]);
        assert!(one.iter().any(|s| steps(s) == ["a", "b"]));
        let two = suggest(&period(), &with_noise(2), &[]);
        assert!(!two.iter().any(|s| steps(s) == ["a", "b"]));
    }

    #[test]
    fn a_wait_of_more_than_three_hours_breaks_the_sequence() {
        let entries: Vec<LifeEntry> = (1..=6)
            .flat_map(|d| day_of(d, &[("a", "08:00", "08:30"), ("b", "11:31", "12:00")]))
            .collect();
        assert!(suggest(&period(), &entries, &[]).is_empty());
    }

    #[test]
    fn a_description_never_repeats_inside_a_sequence() {
        let entries: Vec<LifeEntry> = (1..=6)
            .flat_map(|d| {
                day_of(
                    d,
                    &[
                        ("a", "08:00", "08:30"),
                        ("b", "08:40", "09:00"),
                        ("a", "09:10", "09:30"),
                    ],
                )
            })
            .collect();
        let found = suggest(&period(), &entries, &[]);
        assert!(!found.is_empty());
        assert!(found.iter().all(|s| s.steps.len() == 2));
    }

    #[test]
    fn sleep_over_midnight_does_not_link_to_the_next_days_entries() {
        // Each Day: dinner, then sleep starting 23:00 and ending the next morning.
        // The next Day starts with a bath at 07:30. Sleep -> bath spans two Days, so no sequence.
        let mut entries = vec![];
        for d in 1..=7 {
            entries.push(entry("p", "dinner", &at(d, "21:00"), &at(d, "21:45")));
            entries.push(entry("p", "sleep", &at(d, "23:00"), &at(d + 1, "07:00")));
            entries.push(entry("p", "bath", &at(d + 1, "07:30"), &at(d + 1, "07:45")));
        }
        let found = suggest(&period(), &entries, &[]);
        assert!(found.iter().any(|s| steps(s) == ["dinner", "sleep"]));
        assert!(!found.iter().any(|s| s.steps.contains(&"bath".to_owned())
            && s.steps.contains(&"sleep".to_owned())
            && steps(s).iter().position(|x| *x == "sleep")
                < steps(s).iter().position(|x| *x == "bath")));
    }

    #[test]
    fn entries_without_a_description_or_outside_the_period_are_ignored() {
        let mut entries = office_days(6);
        entries.push(entry("p", "", &at(1, "12:00"), &at(1, "12:30")));
        let narrow = Period::parse("2026-05-01", "2026-05-04", "UTC").unwrap();
        assert!(suggest(&narrow, &entries, &[]).is_empty());
        assert_eq!(suggest(&period(), &entries, &[]).len(), 1);
    }

    #[test]
    fn a_saved_routine_hides_its_own_sequence_and_the_parts_of_it() {
        let saved = vec![vec![
            "Travel To_Office".to_owned(),
            "inside_office".to_owned(),
            "travel from_office".to_owned(),
        ]];
        assert!(suggest(&period(), &office_days(8), &saved).is_empty());
    }

    #[test]
    fn at_most_the_best_few_are_offered() {
        // Twelve independent pairs, each on six Days.
        let entries: Vec<LifeEntry> = (0..12)
            .flat_map(|n| {
                (1..=6).flat_map(move |d| {
                    let (a, b) = (format!("a{n}"), format!("b{n}"));
                    let hour = 6 + n;
                    vec![
                        entry(
                            "p",
                            &a,
                            &at(d, &format!("{hour:02}:00")),
                            &at(d, &format!("{hour:02}:10")),
                        ),
                        entry(
                            "p",
                            &b,
                            &at(d, &format!("{hour:02}:20")),
                            &at(d, &format!("{hour:02}:30")),
                        ),
                    ]
                })
            })
            .collect();
        assert_eq!(suggest(&period(), &entries, &[]).len(), MAX_SUGGESTIONS);
    }

    #[test]
    fn better_covered_sequences_rank_first() {
        let mut entries = office_days(6);
        // x -> y on 9 other Days: 9 * 2 = 18 beats nothing, but 6 * 3 = 18 ties; use 10 Days.
        for d in 10..=19 {
            entries.extend(day_of(
                d,
                &[("x", "10:00", "10:30"), ("y", "10:40", "11:00")],
            ));
        }
        let found = suggest(&period(), &entries, &[]);
        assert_eq!(steps(&found[0]), ["x", "y"]);
        assert_eq!(found[0].days, 10);
        assert_eq!(found.len(), 2);
    }
}
