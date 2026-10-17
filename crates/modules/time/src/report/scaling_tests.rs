//! Tests for `scaling`.

use chrono::{DateTime, Utc};

use super::{
    period::{Period, PeriodKind},
    scaling::*,
};

fn at(text: &str) -> DateTime<Utc> {
    text.parse().unwrap()
}

fn period(tz: &str) -> Period {
    // Monday 2026-10-12 to Sunday 2026-10-18; the Usual's range is 2026-09-14..=2026-10-11.
    Period::new(
        PeriodKind::Week,
        "2026-10-12".parse().unwrap(),
        "2026-10-18".parse().unwrap(),
        tz.parse().unwrap(),
    )
    .unwrap()
}

fn scaling(now: &str, first: Option<&str>) -> Scaling {
    Scaling::new(&period("UTC"), at(now), first.map(at))
}

const LONG_AGO: Option<&str> = Some("2026-01-01T09:00:00Z");

#[test]
fn a_past_period_has_all_its_days_elapsed() {
    assert_eq!(scaling("2026-10-25T10:00:00Z", LONG_AGO).elapsed_days, 7);
}

#[test]
fn the_current_period_counts_days_up_to_and_including_today() {
    assert_eq!(scaling("2026-10-14T10:00:00Z", LONG_AGO).elapsed_days, 3);
    assert_eq!(scaling("2026-10-12T00:00:00Z", LONG_AGO).elapsed_days, 1);
    assert_eq!(scaling("2026-10-18T23:59:00Z", LONG_AGO).elapsed_days, 7);
}

#[test]
fn a_period_that_has_not_started_has_no_days_elapsed() {
    assert_eq!(scaling("2026-10-11T23:59:00Z", LONG_AGO).elapsed_days, 0);
}

#[test]
fn today_is_the_day_in_the_persons_time_zone() {
    // 23:30 UTC on the 14th is already the 15th in Auckland (UTC+13 in October).
    let auckland = Scaling::new(
        &period("Pacific/Auckland"),
        at("2026-10-14T23:30:00Z"),
        LONG_AGO.map(at),
    );
    assert_eq!(auckland.elapsed_days, 4);
}

#[test]
fn full_history_makes_every_day_of_the_usual_available() {
    assert_eq!(scaling("2026-10-25T10:00:00Z", LONG_AGO).available_days, 28);
}

#[test]
fn history_that_starts_inside_the_usual_counts_from_that_day() {
    // First entry on 2026-10-02: the 2nd to the 11th is 10 Days.
    assert_eq!(
        scaling("2026-10-25T10:00:00Z", Some("2026-10-02T23:00:00Z")).available_days,
        10
    );
}

#[test]
fn the_first_entry_day_is_the_day_in_the_persons_time_zone() {
    // 2026-10-01T23:30Z is already 2026-10-02 in Auckland, so one Day fewer than in UTC.
    let first = Some(at("2026-10-01T23:30:00Z"));
    let auckland = Scaling::new(
        &period("Pacific/Auckland"),
        at("2026-10-25T10:00:00Z"),
        first,
    );
    let utc = Scaling::new(&period("UTC"), at("2026-10-25T10:00:00Z"), first);
    assert_eq!((auckland.available_days, utc.available_days), (10, 11));
}

#[test]
fn history_that_starts_after_the_usual_leaves_no_days_available() {
    assert_eq!(
        scaling("2026-10-25T10:00:00Z", Some("2026-10-12T08:00:00Z")).available_days,
        0
    );
}

#[test]
fn no_entries_at_all_leaves_no_days_available() {
    assert_eq!(scaling("2026-10-25T10:00:00Z", None).available_days, 0);
}

#[test]
fn applying_scales_by_elapsed_over_available_rounding_half_up() {
    let s = Scaling {
        elapsed_days: 7,
        available_days: 28,
    };
    let scaled = s.apply(Totals {
        total_s: 100_800,
        entries: 30,
        days: 18,
    });
    assert_eq!(
        (scaled.total_s, scaled.entries, scaled.days),
        (25_200, 8, 5)
    );
}

#[test]
fn applying_divides_by_the_days_available_not_the_whole_usual() {
    let s = Scaling {
        elapsed_days: 7,
        available_days: 10,
    };
    assert_eq!(
        s.apply(Totals {
            total_s: 36_000,
            entries: 0,
            days: 0
        })
        .total_s,
        25_200
    );
}

#[test]
fn applying_with_nothing_available_gives_zero_instead_of_dividing() {
    let s = Scaling {
        elapsed_days: 7,
        available_days: 0,
    };
    assert_eq!(
        s.apply(Totals {
            total_s: 5,
            entries: 5,
            days: 5
        }),
        Totals::default()
    );
}

#[test]
fn a_comparison_needs_elapsed_days_and_history() {
    assert!(
        Scaling {
            elapsed_days: 1,
            available_days: 1
        }
        .is_comparable()
    );
    assert!(
        !Scaling {
            elapsed_days: 0,
            available_days: 28
        }
        .is_comparable()
    );
    assert!(
        !Scaling {
            elapsed_days: 7,
            available_days: 0
        }
        .is_comparable()
    );
}
