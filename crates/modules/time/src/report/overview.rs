//! The headline numbers of a Report.

use std::collections::BTreeSet;

use super::{
    items::Item,
    model::{Overview, OverviewChange},
};

fn count(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

/// Totals over all items. Each item is one entry, so a multi-tag entry is never doubled.
#[must_use]
pub fn overview(items: &[Item<'_>], before: Option<&Overview>) -> Overview {
    let total_s: i64 = items.iter().map(Item::seconds).sum();
    let days: BTreeSet<_> = items
        .iter()
        .flat_map(|i| i.parts.iter().filter(|p| p.seconds > 0).map(|p| p.date))
        .collect();
    let days_tracked = count(days.len());
    let avg_per_day_s = if days_tracked == 0 {
        0
    } else {
        (total_s + days_tracked / 2) / days_tracked
    };
    let mut now = Overview {
        total_s,
        entries: count(items.len()),
        days_tracked,
        avg_per_day_s,
        change: OverviewChange::default(),
    };
    if let Some(before) = before {
        now.change = OverviewChange {
            total_s: now.total_s - before.total_s,
            entries: now.entries - before.entries,
            days_tracked: now.days_tracked - before.days_tracked,
            avg_per_day_s: now.avg_per_day_s - before.avg_per_day_s,
        };
    }
    now
}
