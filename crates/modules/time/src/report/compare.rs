//! The change between a group's stats and the same group's stats in the Previous Period.

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

/// Fills in `change` and `is_new` on each group (and its children) by matching keys
/// against the Previous Period's groups at the same level.
pub fn attach_changes(groups: &mut [Group], previous: &[Group]) {
    for group in groups {
        let before = previous.iter().find(|p| p.key == group.key);
        if let Some(before) = before {
            group.change = Some(change(&group.stats, &before.stats));
        } else {
            group.is_new = true;
        }
        let earlier_children = before.map_or(&[][..], |b| b.children.as_slice());
        attach_changes(&mut group.children, earlier_children);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::stats::Spread;

    fn stats(total_s: i64, avg_s: i64) -> Stats {
        let spread = Spread {
            avg_s,
            min_s: avg_s,
            max_s: avg_s,
        };
        Stats {
            days: 1,
            entries: 1,
            total_s,
            per_day: spread,
            per_entry: spread,
        }
    }

    #[test]
    fn the_change_is_current_minus_previous() {
        let c = change(&stats(5400, 5400), &stats(3600, 3000));
        assert_eq!(c.total_s, 1800);
        assert_eq!(c.per_day.avg_s, 2400);
        assert_eq!(c.days, 0);
    }

    fn group(key: &str, total_s: i64, children: Vec<Group>) -> Group {
        Group {
            key: key.to_owned(),
            label: key.to_owned(),
            stats: stats(total_s, total_s),
            change: None,
            is_new: false,
            daily: vec![],
            children,
            entries: vec![],
        }
    }

    #[test]
    fn a_group_missing_from_the_previous_period_is_new() {
        let mut now = vec![group("a", 60, vec![]), group("b", 60, vec![])];
        attach_changes(&mut now, &[group("a", 30, vec![])]);
        assert_eq!(now[0].change.unwrap().total_s, 30);
        assert!(!now[0].is_new);
        assert!(now[1].is_new && now[1].change.is_none());
    }

    #[test]
    fn children_are_matched_within_their_own_parent() {
        let mut now = vec![group("a", 60, vec![group("x", 60, vec![])])];
        let before = [group("a", 60, vec![group("y", 60, vec![])])];
        attach_changes(&mut now, &before);
        assert!(now[0].children[0].is_new);
    }

    #[test]
    fn a_drop_is_negative() {
        assert_eq!(change(&stats(60, 60), &stats(120, 120)).total_s, -60);
    }
}
