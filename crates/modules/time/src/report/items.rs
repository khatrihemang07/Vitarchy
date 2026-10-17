//! Entries that pass the filter, each cut into its per-Day parts.

use super::{
    day_split::{DayPart, split},
    entry::WindowEntry,
    filter::Filter,
    period::Period,
};

/// A filtered entry with the time it has on each Day of the Period.
#[derive(Debug)]
pub struct Item<'a> {
    /// The entry.
    pub entry: &'a WindowEntry,
    /// Its parts inside the Period, never empty.
    pub parts: Vec<DayPart>,
}

impl Item<'_> {
    /// Seconds inside the Period.
    #[must_use]
    pub fn seconds(&self) -> i64 {
        self.parts.iter().map(|p| p.seconds).sum()
    }
}

/// Keeps the entries that match `filter` and have time inside `period`.
#[must_use]
pub fn prepare<'a>(entries: &'a [WindowEntry], filter: &Filter, period: &Period) -> Vec<Item<'a>> {
    let range = period.utc_range();
    entries
        .iter()
        .filter(|e| filter.matches(e))
        .map(|entry| Item {
            entry,
            parts: split(entry.start, entry.stop, range, period.tz),
        })
        .filter(|item| !item.parts.is_empty())
        .collect()
}
