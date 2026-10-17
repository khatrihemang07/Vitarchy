//! Occurrence matching when a Routine's later Steps run past midnight (`dinner → bath → sleep`).

use super::{MatchKind, Step, entry, occurrences};
use crate::analysis::{Period, lifelog::LifeEntry};

fn step(value: &str) -> Step {
    Step {
        label: value.into(),
        kind: MatchKind::Description,
        value: value.into(),
    }
}

fn evening() -> Vec<Step> {
    ["dinner", "bath", "sleep"].map(step).to_vec()
}

fn e(description: &str, start: &str, end: &str) -> LifeEntry {
    entry("p", description, start, end)
}

fn period(tz: &str) -> Period {
    Period::parse("2026-05-04", "2026-05-06", tz).unwrap()
}

/// dinner and bath on May 4, then `sleep_start` to 07:00 the next morning.
fn night(sleep_start: &str) -> Vec<LifeEntry> {
    vec![
        e("dinner", "2026-05-04T19:00:00Z", "2026-05-04T20:00:00Z"),
        e("bath", "2026-05-04T22:30:00Z", "2026-05-04T22:50:00Z"),
        e("sleep", sleep_start, "2026-05-05T07:00:00Z"),
    ]
}

#[test]
fn sleep_before_and_after_midnight_both_complete_the_evening() {
    for sleep in ["2026-05-04T23:50:00Z", "2026-05-05T00:10:00Z"] {
        let found = occurrences(&period("UTC"), &evening(), &night(sleep));
        assert_eq!(found.len(), 1, "{sleep}");
        assert!(found[0].complete, "{sleep}");
        assert_eq!(found[0].date.to_string(), "2026-05-04");
    }
}

#[test]
fn a_spilled_step_counts_toward_door_to_door_and_the_gap() {
    let found = occurrences(&period("UTC"), &evening(), &night("2026-05-05T00:10:00Z"));
    assert_eq!(found[0].gaps[1], Some(80 * 60));
    assert_eq!(found[0].door_to_door_seconds, Some(12 * 3600));
}

#[test]
fn an_entry_after_the_cutoff_does_not_spill() {
    let found = occurrences(&period("UTC"), &evening(), &night("2026-05-05T07:00:00Z"));
    assert!(!found[0].complete);
    assert!(found[0].steps[2].is_none());
}

#[test]
fn the_cutoff_is_six_in_the_morning() {
    let at = |t: &str| night(&format!("2026-05-05T{t}:00Z"));
    assert!(occurrences(&period("UTC"), &evening(), &at("05:59"))[0].complete);
    assert!(!occurrences(&period("UTC"), &evening(), &at("06:00"))[0].complete);
}

#[test]
fn a_spilled_entry_is_not_also_used_by_the_next_days_occurrence() {
    let mut entries = night("2026-05-05T00:10:00Z");
    entries.push(e("dinner", "2026-05-05T19:00:00Z", "2026-05-05T20:00:00Z"));
    entries.push(e("bath", "2026-05-05T22:30:00Z", "2026-05-05T22:50:00Z"));
    entries.push(e("sleep", "2026-05-06T00:20:00Z", "2026-05-06T07:00:00Z"));
    let found = occurrences(&period("UTC"), &evening(), &entries);
    assert_eq!(found.len(), 2);
    assert!(found.iter().all(|o| o.complete));
    let sleeps: Vec<String> = found
        .iter()
        .map(|o| o.steps[2].as_ref().unwrap().start.to_string())
        .collect();
    assert_eq!(
        sleeps,
        ["2026-05-05 00:10:00 UTC", "2026-05-06 00:20:00 UTC"]
    );
}

#[test]
fn a_day_left_with_only_spilled_entries_is_not_an_occurrence() {
    let found = occurrences(&period("UTC"), &evening(), &night("2026-05-05T00:10:00Z"));
    assert_eq!(found.len(), 1);
}

#[test]
fn nothing_spills_when_no_earlier_step_happened_the_day_before() {
    let entries = vec![e("sleep", "2026-05-05T00:10:00Z", "2026-05-05T07:00:00Z")];
    let found = occurrences(&period("UTC"), &evening(), &entries);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].date.to_string(), "2026-05-05");
    assert!(!found[0].complete);
}

#[test]
fn several_steps_can_run_past_midnight() {
    let entries = vec![
        e("dinner", "2026-05-04T22:00:00Z", "2026-05-04T23:00:00Z"),
        e("bath", "2026-05-05T00:05:00Z", "2026-05-05T00:25:00Z"),
        e("sleep", "2026-05-05T00:30:00Z", "2026-05-05T07:00:00Z"),
    ];
    let found = occurrences(&period("UTC"), &evening(), &entries);
    assert_eq!(found.len(), 1);
    assert!(found[0].complete);
}

#[test]
fn the_last_day_of_the_period_can_spill_into_entries_after_it() {
    let period = Period::parse("2026-05-04", "2026-05-04", "UTC").unwrap();
    let found = occurrences(&period, &evening(), &night("2026-05-05T00:10:00Z"));
    assert!(found[0].complete);
}

#[test]
fn the_first_days_early_entries_belong_to_the_evening_before_it() {
    let period = Period::parse("2026-05-05", "2026-05-05", "UTC").unwrap();
    let found = occurrences(&period, &evening(), &night("2026-05-05T00:10:00Z"));
    assert!(found.is_empty());
}

#[test]
fn the_cutoff_is_local_time_in_a_zone_with_a_dst_change() {
    // London springs forward on 29 March 2026: 05:30 UTC is 06:30 BST (after the cutoff),
    // 04:50 UTC is 05:50 BST (before it).
    let period = Period::parse("2026-03-28", "2026-03-29", "Europe/London").unwrap();
    let night = |sleep: &str| {
        vec![
            e("dinner", "2026-03-28T19:00:00Z", "2026-03-28T20:00:00Z"),
            e("bath", "2026-03-28T22:30:00Z", "2026-03-28T22:50:00Z"),
            e("sleep", sleep, "2026-03-29T09:00:00Z"),
        ]
    };
    let late = occurrences(&period, &evening(), &night("2026-03-29T05:30:00Z"));
    assert!(!late[0].complete);
    let early = occurrences(&period, &evening(), &night("2026-03-29T04:50:00Z"));
    assert!(early[0].complete);
}

#[test]
fn the_cutoff_is_local_time_when_the_clocks_go_back() {
    // London falls back on 25 October 2026: 05:30 UTC is 05:30 GMT, 06:30 UTC is 06:30 GMT.
    let period = Period::parse("2026-10-24", "2026-10-25", "Europe/London").unwrap();
    let night = |sleep: &str| {
        vec![
            e("dinner", "2026-10-24T18:00:00Z", "2026-10-24T19:00:00Z"),
            e("bath", "2026-10-24T21:30:00Z", "2026-10-24T21:50:00Z"),
            e("sleep", sleep, "2026-10-25T09:00:00Z"),
        ]
    };
    assert!(occurrences(&period, &evening(), &night("2026-10-25T05:30:00Z"))[0].complete);
    assert!(!occurrences(&period, &evening(), &night("2026-10-25T06:30:00Z"))[0].complete);
}
