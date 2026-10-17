//! Tests for `compare`; the helpers are shared with the Usual's tests.

use super::{
    compare::*,
    model::Group,
    stats::{Spread, Stats},
    usual::UsualComparison,
};

pub(super) fn spread(avg_s: i64, min_s: i64, max_s: i64) -> Spread {
    Spread {
        avg_s,
        min_s,
        max_s,
    }
}

/// Every field distinct, so a change that reads the wrong field cannot match.
pub(super) fn stats(
    days: i64,
    entries: i64,
    total_s: i64,
    per_day: Spread,
    per_entry: Spread,
) -> Stats {
    Stats {
        days,
        entries,
        total_s,
        per_day,
        per_entry,
    }
}

pub(super) fn group(key: &str, stats: Stats, children: Vec<Group>) -> Group {
    Group {
        key: key.to_owned(),
        label: key.to_owned(),
        stats,
        vs_previous: Comparison::default(),
        vs_usual: UsualComparison::default(),
        daily: vec![],
        children,
        entries: vec![],
    }
}

pub(super) fn simple(total_s: i64) -> Stats {
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
    attach_previous(&mut now, &[group("a", simple(30), vec![])]);
    assert_eq!(now[0].vs_previous.change.unwrap().total_s, 30);
    assert!(!now[0].vs_previous.is_new);
    assert!(now[1].vs_previous.is_new && now[1].vs_previous.change.is_none());
}

#[test]
fn a_matched_child_gets_its_own_change() {
    let mut now = vec![group(
        "a",
        simple(100),
        vec![group("x", simple(70), vec![])],
    )];
    let before = [group("a", simple(60), vec![group("x", simple(50), vec![])])];
    attach_previous(&mut now, &before);
    assert_eq!(now[0].vs_previous.change.unwrap().total_s, 40);
    let child = &now[0].children[0];
    assert!(!child.vs_previous.is_new);
    assert_eq!(child.vs_previous.change.unwrap().total_s, 20);
}

#[test]
fn children_are_matched_within_their_own_parent() {
    let mut now = vec![group("a", simple(60), vec![group("x", simple(60), vec![])])];
    let before = [
        group("a", simple(60), vec![group("y", simple(60), vec![])]),
        group("b", simple(60), vec![group("x", simple(60), vec![])]),
    ];
    attach_previous(&mut now, &before);
    assert!(now[0].children[0].vs_previous.is_new);
}

#[test]
fn a_new_parent_makes_all_its_children_new() {
    let mut now = vec![group("a", simple(60), vec![group("x", simple(60), vec![])])];
    attach_previous(&mut now, &[]);
    assert!(now[0].vs_previous.is_new && now[0].children[0].vs_previous.is_new);
}
