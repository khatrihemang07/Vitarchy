//! Linked sequences of descriptions inside one Day.
//!
//! Within one Day, entries are taken in start order (consecutive entries with the same
//! description count as one). A sequence links descriptions that follow each other with at most
//! [`MAX_BETWEEN`] unrelated entries in between and no more than [`MAX_GAP`] of time between one
//! ending and the next starting. Descriptions are compared by their folded form.

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{DateTime, Duration, NaiveDate, Utc};

use super::spill::{by_day, dates, is_early, next_day};
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
    /// Starts on the next Day, in the early hours (see [`spill`](super::spill)).
    spilled: bool,
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

/// Each Day's described entries in start order, merged into runs. An entry belongs to the Day it
/// starts in, and the next Day's early-hours entries follow as `spilled` runs, so an evening
/// that ends after midnight is one chain.
fn runs_by_day<'a>(period: &Period, entries: &'a [LifeEntry]) -> BTreeMap<NaiveDate, Vec<Run<'a>>> {
    let tz = period.tz();
    let (first, last) = dates(period);
    let days = by_day(
        period,
        entries
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.description.is_empty()),
    );
    days.range(first..=last)
        .map(|(date, own)| {
            let next = days.get(&next_day(*date)).map_or(&[][..], Vec::as_slice);
            let early = next
                .iter()
                .filter(|s| is_early(tz, s.entry))
                .map(|s| (s.entry, true));
            let mut runs: Vec<Run> = vec![];
            for (entry, spilled) in own.iter().map(|s| (s.entry, false)).chain(early) {
                match runs.last_mut() {
                    Some(run) if run.key == entry.description_key => {
                        run.end = run.end.max(entry.end);
                    }
                    _ => runs.push(Run {
                        key: &entry.description_key,
                        start: entry.start,
                        end: entry.end,
                        spilled,
                    }),
                }
            }
            (*date, runs)
        })
        .collect()
}

/// Every linked sequence of 2 to [`MAX_SUGGESTED_STEPS`] distinct descriptions in one Day's runs.
/// A chain must begin on the Day itself, so one made only of next-morning runs counts for that
/// morning's Day and never twice.
fn day_chains<'a>(runs: &[Run<'a>]) -> HashSet<Chain<'a>> {
    let mut found = HashSet::new();
    for first in (0..runs.len()).filter(|i| !runs[*i].spilled) {
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
