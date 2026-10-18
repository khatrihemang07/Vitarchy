//! Tests for `away_rules`.

use chrono::Duration;

use super::away_rules::*;

const HOUR: i64 = 3600;

fn away(no_input_secs: i64, watching_secs: i64) -> i64 {
    stretch_away(
        Duration::seconds(no_input_secs),
        Duration::seconds(watching_secs),
        &AwayRules::current(),
    )
    .num_seconds()
}

#[test]
fn current_rules_are_an_hour_and_two_hours() {
    let rules = AwayRules::current();
    assert_eq!(rules.away_after.num_minutes(), 60);
    assert_eq!(rules.watching_cap.num_hours(), 2);
}

#[test]
fn fifty_nine_fifty_nine_of_no_input_is_not_away() {
    assert_eq!(away(3599, 0), 0);
}

#[test]
fn sixty_minutes_of_no_input_is_all_away() {
    assert_eq!(away(3600, 0), HOUR);
}

#[test]
fn no_input_is_none() {
    assert_eq!(away(0, 0), 0);
}

#[test]
fn an_hour_and_a_half_of_watching_stays_active() {
    assert_eq!(away(5400, 5400), 0);
}

#[test]
fn only_the_part_beyond_two_hours_of_watching_is_away() {
    assert_eq!(away(12_600, 12_600), 5400);
}

#[test]
fn a_tail_under_the_threshold_after_the_cap_is_not_away() {
    assert_eq!(away(9000, 9000), 0);
}

#[test]
fn only_the_watching_part_of_a_stretch_is_kept_active() {
    assert_eq!(away(3 * HOUR, HOUR), 2 * HOUR);
    assert_eq!(away(3 * HOUR, 3 * HOUR), HOUR);
}

#[test]
fn a_lower_threshold_counts_shorter_stretches() {
    let rules = AwayRules::current().with_away_after(Duration::minutes(5));
    let away = stretch_away(Duration::seconds(600), Duration::zero(), &rules);
    assert_eq!(away.num_seconds(), 600);
}
