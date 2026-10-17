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

    fn spread(avg_s: i64, min_s: i64, max_s: i64) -> Spread {
        Spread {
            avg_s,
            min_s,
            max_s,
        }
    }

    /// Every field distinct, so a change that reads the wrong field cannot match.
    fn stats(days: i64, entries: i64, total_s: i64, per_day: Spread, per_entry: Spread) -> Stats {
        Stats {
            days,
            entries,
            total_s,
            per_day,
            per_entry,
        }
    }

    fn group(key: &str, stats: Stats, children: Vec<Group>) -> Group {
        Group {
            key: key.to_owned(),
            label: key.to_owned(),
            stats,
            change: None,
            is_new: false,
            daily: vec![],
            children,
            entries: vec![],
        }
    }

    fn simple(total_s: i64) -> Stats {
        stats(
            1,
            1,
            total_s,
            spread(total_s, total_s, total_s),
            spread(total_s, total_s, total_s),
        )
    }

    #[test]
    fn each_figure_changes_by_its_own_difference() {
        let now = stats(5, 9, 10_000, spread(700, 300, 1100), spread(450, 60, 2000));
        let before = stats(2, 4, 6_000, spread(500, 100, 900), spread(400, 20, 1500));
        assert_eq!(
            change(&now, &before),
            StatsChange {
                days: 3,
                entries: 5,
                total_s: 4_000,
                per_day: SpreadChange {
                    avg_s: 200,
                    min_s: 200,
                    max_s: 200,
                },
                per_entry: SpreadChange {
                    avg_s: 50,
                    min_s: 40,
                    max_s: 500,
                },
            }
        );
    }

    #[test]
    fn the_per_day_and_per_entry_spreads_do_not_swap() {
        let now = stats(1, 1, 1, spread(10, 20, 30), spread(1, 2, 3));
        let before = stats(1, 1, 1, spread(1, 1, 1), spread(1, 1, 1));
        let c = change(&now, &before);
        assert_eq!(
            (c.per_day.avg_s, c.per_day.min_s, c.per_day.max_s),
            (9, 19, 29)
        );
        assert_eq!(
            (c.per_entry.avg_s, c.per_entry.min_s, c.per_entry.max_s),
            (0, 1, 2)
        );
    }

    #[test]
    fn a_drop_is_negative() {
        assert_eq!(change(&simple(60), &simple(120)).total_s, -60);
    }

    #[test]
    fn a_group_missing_from_the_previous_period_is_new() {
        let mut now = vec![
            group("a", simple(60), vec![]),
            group("b", simple(60), vec![]),
        ];
        attach_changes(&mut now, &[group("a", simple(30), vec![])]);
        assert_eq!(now[0].change.unwrap().total_s, 30);
        assert!(!now[0].is_new);
        assert!(now[1].is_new && now[1].change.is_none());
    }

    #[test]
    fn a_matched_child_gets_its_own_change() {
        let mut now = vec![group(
            "a",
            simple(100),
            vec![group("x", simple(70), vec![])],
        )];
        let before = [group("a", simple(60), vec![group("x", simple(50), vec![])])];
        attach_changes(&mut now, &before);
        assert_eq!(now[0].change.unwrap().total_s, 40);
        let child = &now[0].children[0];
        assert!(!child.is_new);
        assert_eq!(child.change.unwrap().total_s, 20);
    }

    #[test]
    fn children_are_matched_within_their_own_parent() {
        let mut now = vec![group("a", simple(60), vec![group("x", simple(60), vec![])])];
        let before = [
            group("a", simple(60), vec![group("y", simple(60), vec![])]),
            group("b", simple(60), vec![group("x", simple(60), vec![])]),
        ];
        attach_changes(&mut now, &before);
        assert!(now[0].children[0].is_new);
    }

    #[test]
    fn a_new_parent_makes_all_its_children_new() {
        let mut now = vec![group("a", simple(60), vec![group("x", simple(60), vec![])])];
        attach_changes(&mut now, &[]);
        assert!(now[0].is_new && now[0].children[0].is_new);
    }
}
