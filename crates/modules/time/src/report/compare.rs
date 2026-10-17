//! The change between a group's stats and a reference's stats, matched by key per level.

use serde::Serialize;
use utoipa::ToSchema;

use super::{
    model::Group,
    stats::{Spread, Stats},
};

/// Signed difference of a [`Spread`]: current minus previous, in seconds.
// The `_s` suffix is the API's unit convention (`total_s`), not redundancy.
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, ToSchema)]
pub struct SpreadChange {
    /// Change in the average.
    pub avg_s: i64,
    /// Change in the shortest.
    pub min_s: i64,
    /// Change in the longest.
    pub max_s: i64,
}

/// Signed difference of [`Stats`]: current minus previous.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, ToSchema)]
pub struct StatsChange {
    /// Change in the number of Days (a count, not seconds).
    pub days: i64,
    /// Change in the number of entries (a count, not seconds).
    pub entries: i64,
    /// Change in the total, in seconds.
    pub total_s: i64,
    /// Change in the per-Day spread.
    pub per_day: SpreadChange,
    /// Change in the per-entry spread.
    pub per_entry: SpreadChange,
}

fn spread_change(now: Spread, before: Spread) -> SpreadChange {
    SpreadChange {
        avg_s: now.avg_s - before.avg_s,
        min_s: now.min_s - before.min_s,
        max_s: now.max_s - before.max_s,
    }
}

/// Current minus previous, stat by stat.
#[must_use]
pub fn change(now: &Stats, before: &Stats) -> StatsChange {
    StatsChange {
        days: now.days - before.days,
        entries: now.entries - before.entries,
        total_s: now.total_s - before.total_s,
        per_day: spread_change(now.per_day, before.per_day),
        per_entry: spread_change(now.per_entry, before.per_entry),
    }
}

/// How a group compares with a reference: the Previous Period's same group, or the Usual's.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, ToSchema)]
pub struct Comparison {
    /// Current minus reference; absent when `is_new`.
    pub change: Option<StatsChange>,
    /// True when the group has no time in the reference.
    pub is_new: bool,
}

impl Comparison {
    /// Compares `now` with `reference`; no reference means the group is new.
    #[must_use]
    pub fn against(now: &Stats, reference: Option<&Stats>) -> Self {
        Self {
            change: reference.map(|r| change(now, r)),
            is_new: reference.is_none(),
        }
    }
}

/// Walks `groups` beside the `reference` groups at the same level (matched by key, children
/// within their own parent) and calls `apply` with each group and its match.
pub fn walk_matches(
    groups: &mut [Group],
    reference: &[Group],
    apply: &mut impl FnMut(&mut Group, Option<&Group>),
) {
    for group in groups {
        let matched = reference.iter().find(|r| r.key == group.key);
        apply(group, matched);
        let reference_children = matched.map_or(&[][..], |m| m.children.as_slice());
        walk_matches(&mut group.children, reference_children, apply);
    }
}

/// Fills `vs_previous` on each group (and its children) from the Previous Period's groups.
pub fn attach_previous(groups: &mut [Group], previous: &[Group]) {
    walk_matches(groups, previous, &mut |group, before| {
        group.vs_previous = Comparison::against(&group.stats, before.map(|b| &b.stats));
    });
}
