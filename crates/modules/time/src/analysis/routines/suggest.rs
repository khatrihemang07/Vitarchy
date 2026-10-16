//! Finds Routines the person has not saved yet: sequences of descriptions that recur in order.
//!
//! Chains come from [`chains`](super::chains): 2 to [`MAX_SUGGESTED_STEPS`] distinct descriptions
//! seen in order on at least [`MIN_DAYS`] Days. A chain that is a part of a better one (or of a
//! saved Routine) is dropped, and [`redundancy`](super::redundancy) removes near-duplicates. The
//! rest are ranked by Days covered times length. Descriptions are matched ignoring case and
//! spaces; each is shown in the spelling used most often.

use std::collections::HashMap;

use serde::Serialize;
use utoipa::ToSchema;

pub use super::chains::MAX_SUGGESTED_STEPS;
use super::chains::{Chain, days_per_chain, described_in};
use super::draft::MAX_VALUE_CHARS;
use super::redundancy::drop_redundant;
use crate::analysis::Period;
use crate::analysis::lifelog::{LifeEntry, MIN_DAYS, fold};

/// The most suggestions offered at once: past these the list is noise.
pub const MAX_SUGGESTIONS: usize = 6;

/// A recurring sequence of descriptions, offered to become a Routine.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct Suggestion {
    /// The descriptions in order.
    pub steps: Vec<String>,
    /// Days on which the whole sequence happened.
    pub days: i64,
}

/// Suggestions for the Period, best first, leaving out what `saved` (description sequences of
/// existing Routines) already covers.
#[must_use]
pub fn suggest(period: &Period, entries: &[LifeEntry], saved: &[Vec<String>]) -> Vec<Suggestion> {
    let candidates: Vec<(Chain, usize)> = days_per_chain(period, entries)
        .into_iter()
        .filter(|(chain, days)| *days >= MIN_DAYS && chain.iter().all(|key| fits_a_step(key)))
        .collect();
    let saved: Vec<Vec<String>> = saved
        .iter()
        .map(|steps| steps.iter().map(|s| fold(s)).collect())
        .collect();
    let mut ranked: Vec<(Chain, usize)> = drop_redundant(candidates)
        .into_iter()
        .filter(|(chain, _)| !saved.iter().any(|s| is_part_of(chain, s)))
        .collect();
    ranked.sort_by(|(a, a_days), (b, b_days)| {
        (b_days * b.len())
            .cmp(&(a_days * a.len()))
            .then_with(|| a.cmp(b))
    });
    let mut kept: Vec<(Chain, usize)> = vec![];
    for (chain, days) in ranked {
        if !kept.iter().any(|(better, _)| is_part_of(&chain, better)) {
            kept.push((chain, days));
        }
    }
    let spellings = commonest_spellings(period, entries);
    kept.into_iter()
        .take(MAX_SUGGESTIONS)
        .map(|(chain, days)| Suggestion {
            steps: chain
                .into_iter()
                .map(|key| spellings.get(key).copied().unwrap_or(key).to_owned())
                .collect(),
            days: i64::try_from(days).unwrap_or(i64::MAX),
        })
        .collect()
}

/// Whether a description is short enough to become a Step's match (otherwise it could not be saved).
fn fits_a_step(description: &str) -> bool {
    description.chars().count() <= MAX_VALUE_CHARS
}

/// Whether `part` appears in `whole` in the same order (not necessarily next to each other).
fn is_part_of<A: PartialEq<B>, B>(part: &[A], whole: &[B]) -> bool {
    let mut rest = whole.iter();
    part.iter().all(|p| rest.any(|w| p == w))
}

/// For each folded description, the spelling that most entries used (the first alphabetically on a tie).
fn commonest_spellings<'a>(period: &Period, entries: &'a [LifeEntry]) -> HashMap<&'a str, &'a str> {
    let mut counts: HashMap<&str, HashMap<&str, usize>> = HashMap::new();
    for entry in described_in(period, entries) {
        *counts
            .entry(&entry.description_key)
            .or_default()
            .entry(&entry.description)
            .or_default() += 1;
    }
    counts
        .into_iter()
        .filter_map(|(key, spellings)| {
            let best = spellings
                .into_iter()
                .max_by(|(a, a_n), (b, b_n)| a_n.cmp(b_n).then_with(|| b.cmp(a)))?;
            Some((key, best.0))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::entry;
    use super::*;

    fn period() -> Period {
        Period::parse("2026-05-01", "2026-07-31", "UTC").unwrap()
    }

    /// `clock` on the `day`th day counting from 2026-05-01 (past 31 it runs into June).
    fn at(day: u32, clock: &str) -> String {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()
            + chrono::Duration::days(i64::from(day) - 1);
        format!("{date}T{clock}:00Z")
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

    #[test]
    fn descriptions_that_differ_only_in_case_are_one_and_the_commonest_spelling_is_shown() {
        let mut entries = vec![];
        for d in 1..=6 {
            let (a, b) = if d <= 2 {
                ("Dinner", "Bath")
            } else {
                ("dinner", "bath")
            };
            entries.extend(day_of(d, &[(a, "20:00", "20:30"), (b, "21:00", "21:15")]));
        }
        let found = suggest(&period(), &entries, &[]);
        assert_eq!(found.len(), 1);
        assert_eq!(
            (steps(&found[0]), found[0].days),
            (vec!["dinner", "bath"], 6)
        );
    }

    #[test]
    fn a_description_longer_than_a_step_can_match_is_never_suggested() {
        let long = "x".repeat(201);
        let entries: Vec<LifeEntry> = (1..=6)
            .flat_map(|d| {
                day_of(
                    d,
                    &[("a", "08:00", "08:30"), (long.as_str(), "09:00", "09:30")],
                )
            })
            .collect();
        assert!(suggest(&period(), &entries, &[]).is_empty());
    }

    /// 48 Days shaped like the seeded data: the office trip on 40 of them (21 with a short wait
    /// before dinner), workouts on some, dinner, bath and sleep every Day.
    fn seeded_shape() -> Vec<LifeEntry> {
        let mut entries = vec![];
        for d in 1..=48 {
            let office = d % 6 != 0;
            if office {
                let back_ends = if d % 2 == 0 && d % 5 != 0 {
                    "18:30"
                } else {
                    "17:00"
                };
                entries.extend(day_of(
                    d,
                    &[
                        ("travel to_office", "08:00", "08:30"),
                        ("inside_office", "08:40", "16:30"),
                        ("travel from_office", "16:35", back_ends),
                    ],
                ));
            }
            if d % 5 < 2 {
                entries.extend(day_of(d, &[("workout", "19:45", "20:25")]));
            }
            entries.extend(day_of(
                d,
                &[("dinner", "21:05", "21:40"), ("bath", "22:15", "22:30")],
            ));
            entries.push(entry("p", "sleep", &at(d, "23:10"), &at(d + 1, "06:00")));
        }
        entries
    }

    #[test]
    fn near_duplicate_chains_collapse_to_the_ones_that_say_something_new() {
        let found = suggest(&period(), &seeded_shape(), &[]);
        let mut chains: Vec<Vec<&str>> = found.iter().map(steps).collect();
        chains.sort();
        assert_eq!(
            chains,
            [
                vec!["dinner", "bath", "sleep"],
                vec!["travel to_office", "inside_office", "travel from_office"],
            ]
        );
    }

    #[test]
    fn a_longer_chain_is_kept_when_it_happens_almost_as_often_and_replaces_the_shorter() {
        // a -> b on 10 Days; a -> b -> c on 9 of them (9 >= 80% of 10).
        let mut entries = vec![];
        for d in 1..=10 {
            let mut parts = vec![("a", "08:00", "08:30"), ("b", "09:00", "09:30")];
            if d <= 9 {
                parts.push(("c", "10:00", "10:30"));
            }
            entries.extend(day_of(d, &parts));
        }
        let found = suggest(&period(), &entries, &[]);
        assert_eq!(
            found.iter().map(steps).collect::<Vec<_>>(),
            [vec!["a", "b", "c"]]
        );
    }

    #[test]
    fn a_longer_chain_that_is_much_rarer_is_dropped_in_favour_of_the_shorter() {
        // a -> b on 10 Days; a -> b -> c on 6 of them (6 < 80% of 10).
        let mut entries = vec![];
        for d in 1..=10 {
            let mut parts = vec![("a", "08:00", "08:30"), ("b", "09:00", "09:30")];
            if d <= 6 {
                parts.push(("c", "10:00", "10:30"));
            }
            entries.extend(day_of(d, &parts));
        }
        let found = suggest(&period(), &entries, &[]);
        let chains: Vec<_> = found.iter().map(steps).collect();
        assert!(chains.contains(&vec!["a", "b"]));
        assert!(!chains.contains(&vec!["a", "b", "c"]));
    }
}
