//! Linked sequences of descriptions inside one Day.
//!
//! Within one Day, entries are taken in start order (consecutive entries with the same
//! description count as one). A sequence links descriptions that follow each other with at most
//! [`MAX_BETWEEN`] unrelated entries in between and no more than [`MAX_GAP`] of time between one
//! ending and the next starting. Descriptions are compared by their folded form.

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{DateTime, Duration, NaiveDate, Utc};

use crate::analysis::Period;
use crate::analysis::lifelog::LifeEntry;

/// The most Steps a suggested Routine has.
pub const MAX_SUGGESTED_STEPS: usize = 5;
/// How many unrelated entries may sit between two linked ones.
const MAX_BETWEEN: usize = 1;
/// The longest wait between one entry ending and the next starting.
const MAX_GAP: Duration = Duration::hours(3);

/// A chain of folded descriptions.
pub(super) type Chain<'a> = Vec<&'a str>;

/// Consecutive entries with one description, merged.
struct Run<'a> {
    key: &'a str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

/// The described entries that start inside the Period.
pub(super) fn described_in<'a>(
    period: &Period,
    entries: &'a [LifeEntry],
) -> impl Iterator<Item = &'a LifeEntry> {
    let (from, to) = (period.start(), period.end());
    entries
        .iter()
        .filter(move |e| !e.description.is_empty() && e.start >= from && e.start < to)
}

/// How many Days each linked chain of 2 to [`MAX_SUGGESTED_STEPS`] distinct descriptions occurs on.
pub(super) fn days_per_chain<'a>(
    period: &Period,
    entries: &'a [LifeEntry],
) -> HashMap<Chain<'a>, usize> {
    let mut counts: HashMap<Chain, usize> = HashMap::new();
    for runs in runs_by_day(period, entries).values() {
        for chain in day_chains(runs) {
            *counts.entry(chain).or_default() += 1;
        }
    }
    counts
}

/// Each Day's described entries in start order, merged into runs. An entry belongs to the Day it starts in.
fn runs_by_day<'a>(period: &Period, entries: &'a [LifeEntry]) -> BTreeMap<NaiveDate, Vec<Run<'a>>> {
    let mut days: BTreeMap<NaiveDate, Vec<&LifeEntry>> = BTreeMap::new();
    for entry in described_in(period, entries) {
        let date = entry.start.with_timezone(&period.tz()).date_naive();
        days.entry(date).or_default().push(entry);
    }
    days.into_iter()
        .map(|(date, mut day)| {
            day.sort_by_key(|e| e.start);
            let mut runs: Vec<Run> = vec![];
            for entry in day {
                match runs.last_mut() {
                    Some(run) if run.key == entry.description_key => {
                        run.end = run.end.max(entry.end);
                    }
                    _ => runs.push(Run {
                        key: &entry.description_key,
                        start: entry.start,
                        end: entry.end,
                    }),
                }
            }
            (date, runs)
        })
        .collect()
}

/// Every linked sequence of 2 to [`MAX_SUGGESTED_STEPS`] distinct descriptions in one Day's runs.
fn day_chains<'a>(runs: &[Run<'a>]) -> HashSet<Chain<'a>> {
    let mut found = HashSet::new();
    for first in 0..runs.len() {
        extend(runs, &[first], &mut found);
    }
    found
}

fn extend<'a>(runs: &[Run<'a>], chain: &[usize], found: &mut HashSet<Chain<'a>>) {
    let last = chain[chain.len() - 1];
    let reach = (last + 1 + MAX_BETWEEN).min(runs.len() - 1);
    for next in last + 1..=reach {
        let linked = runs[next].start - runs[last].end <= MAX_GAP
            && !chain.iter().any(|i| runs[*i].key == runs[next].key);
        if !linked {
            continue;
        }
        let longer: Vec<usize> = chain.iter().copied().chain([next]).collect();
        found.insert(longer.iter().map(|i| runs[*i].key).collect());
        if longer.len() < MAX_SUGGESTED_STEPS {
            extend(runs, &longer, found);
        }
    }
}
