//! Tests for `usual`.

use super::{
    compare_tests::{group, simple, spread, stats},
    model::{Overview, OverviewChange},
    scaling::Scaling,
    usual::*,
};

/// A Usual with all 28 Days of history, compared with `elapsed_days` Days of the Period.
fn over(elapsed_days: i64) -> Scaling {
    Scaling {
        elapsed_days,
        available_days: 28,
    }
}

/// 18 Days, 30 entries, 28 hours: easy to scale by hand.
fn month_of_usual() -> crate::report::stats::Stats {
    stats(
        18,
        30,
        100_800,
        spread(5600, 600, 9000),
        spread(3360, 60, 7200),
    )
}

#[test]
fn a_day_is_compared_with_a_28th_of_the_usual() {
    let s = scaled(&month_of_usual(), &over(1));
    assert_eq!((s.total_s, s.days, s.entries), (3600, 1, 1));
}

#[test]
fn a_week_is_compared_with_a_quarter_of_the_usual_rounded() {
    let s = scaled(&month_of_usual(), &over(7));
    assert_eq!(s.total_s, 25_200);
    assert_eq!((s.days, s.entries), (5, 8)); // 4.5 and 7.5 round up
}

#[test]
fn a_31_day_month_is_compared_with_more_than_the_usual() {
    let s = scaled(&month_of_usual(), &over(31));
    assert_eq!((s.total_s, s.days, s.entries), (111_600, 20, 33));
}

#[test]
fn a_custom_period_scales_by_its_own_length() {
    assert_eq!(scaled(&month_of_usual(), &over(10)).total_s, 36_000);
}

#[test]
fn scaling_leaves_the_spreads_alone() {
    let s = scaled(&month_of_usual(), &over(7));
    assert_eq!(s.per_day, month_of_usual().per_day);
    assert_eq!(s.per_entry, month_of_usual().per_entry);
}

#[test]
fn a_group_absent_from_the_usual_is_new() {
    let mut now = vec![
        group("a", simple(60), vec![]),
        group("b", simple(60), vec![]),
    ];
    attach_usual(&mut now, &[group("a", simple(2800), vec![])], &over(1));
    assert!(!now[0].vs_usual.is_new);
    assert!(now[1].vs_usual.is_new);
    assert!(now[1].vs_usual.change.is_none() && now[1].vs_usual.usual.is_none());
}

#[test]
fn a_day_is_compared_per_day_with_the_usual_average() {
    // 6h28m today against a Usual of 7h10m on average over 18 Days: 42 minutes less.
    let today = stats(
        1,
        2,
        23_280,
        spread(23_280, 23_280, 23_280),
        spread(11_640, 8000, 15_280),
    );
    let usual = stats(
        18,
        30,
        18 * 25_800,
        spread(25_800, 3600, 40_000),
        spread(9000, 60, 20_000),
    );
    let mut now = vec![group("a", today, vec![])];
    attach_usual(&mut now, &[group("a", usual, vec![])], &over(1));
    let vs = now[0].vs_usual;
    assert_eq!(vs.change.unwrap().per_day.avg_s, -42 * 60);
    assert_eq!(vs.change.unwrap().per_day.max_s, 23_280 - 40_000);
    let figures = vs.usual.unwrap();
    assert_eq!(figures.days, 18);
    assert_eq!(figures.per_day, spread(25_800, 3600, 40_000));
    assert_eq!(figures.per_entry, spread(9000, 60, 20_000));
}

#[test]
fn totals_are_compared_with_the_scaled_usual() {
    let mut now = vec![group("a", simple(5000), vec![])];
    attach_usual(&mut now, &[group("a", month_of_usual(), vec![])], &over(7));
    assert_eq!(now[0].vs_usual.change.unwrap().total_s, 5000 - 25_200);
}

#[test]
fn children_are_matched_within_their_own_parent_in_the_usual() {
    let mut now = vec![group("a", simple(60), vec![group("x", simple(60), vec![])])];
    let usual = [
        group("a", simple(60), vec![group("y", simple(60), vec![])]),
        group("b", simple(60), vec![group("x", simple(60), vec![])]),
    ];
    attach_usual(&mut now, &usual, &over(1));
    assert!(!now[0].vs_usual.is_new);
    assert!(now[0].children[0].vs_usual.is_new);
}

#[test]
fn a_matched_child_is_compared_with_the_scaled_usual_child() {
    let mut now = vec![group("a", simple(10), vec![group("x", simple(10), vec![])])];
    let usual = [group(
        "a",
        month_of_usual(),
        vec![group("x", month_of_usual(), vec![])],
    )];
    attach_usual(&mut now, &usual, &over(28));
    assert_eq!(
        now[0].children[0].vs_usual.change.unwrap().total_s,
        10 - 100_800
    );
}

fn overview(total_s: i64, entries: i64, days_tracked: i64, avg_per_day_s: i64) -> Overview {
    Overview {
        total_s,
        entries,
        days_tracked,
        avg_per_day_s,
        usual_days: 0,
        vs_previous: OverviewChange::default(),
        vs_usual: OverviewChange::default(),
    }
}

#[test]
fn the_overview_scales_total_entries_and_days_but_not_the_average() {
    let usual = overview(100_800, 30, 18, 5600);
    let change = overview_change(&overview(30_000, 9, 4, 7500), &usual, &over(7));
    assert_eq!(
        change,
        OverviewChange {
            total_s: 30_000 - 25_200,
            entries: 9 - 8,
            days_tracked: 4 - 5,
            avg_per_day_s: 7500 - 5600,
        }
    );
}

#[test]
fn a_short_history_scales_by_the_days_available() {
    let short = Scaling {
        elapsed_days: 7,
        available_days: 14,
    };
    // Half the available Days have passed in the Period: half the Usual's total.
    assert_eq!(scaled(&month_of_usual(), &short).total_s, 50_400);
}

#[test]
fn a_period_that_has_not_started_shows_no_change_in_its_totals() {
    let mut now = vec![group("a", simple(5000), vec![])];
    attach_usual(&mut now, &[group("a", month_of_usual(), vec![])], &over(0));
    let vs = now[0].vs_usual;
    assert!(!vs.is_new);
    assert_eq!(vs.change.unwrap().total_s, 0);
    assert_eq!(vs.change.unwrap().entries, 0);
}

#[test]
fn the_overview_shows_no_change_without_elapsed_days_or_history() {
    let usual = overview(100_800, 30, 18, 5600);
    let now = overview(30_000, 9, 4, 7500);
    let unstarted = overview_change(&now, &usual, &over(0));
    let no_history = Scaling {
        elapsed_days: 7,
        available_days: 0,
    };
    assert_eq!(unstarted, OverviewChange::default());
    assert_eq!(
        overview_change(&now, &usual, &no_history),
        OverviewChange::default()
    );
}

#[test]
fn usual_figures_are_read_from_a_groups_stats() {
    let figures = UsualFigures::from(&month_of_usual());
    assert_eq!(figures.days, 18);
    assert_eq!(figures.per_day, spread(5600, 600, 9000));
    assert_eq!(figures.per_entry, spread(3360, 60, 7200));
}
