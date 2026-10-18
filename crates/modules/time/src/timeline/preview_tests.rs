//! Tests for `preview`.

use chrono::DateTime;

use super::preview::*;
use crate::activity::{Activity, Extras, Source};

const HOUR: i64 = 3600;
const DAY: i64 = 24 * HOUR;

fn row(source: Source, start: i64, length: i64, no_input: f64) -> Activity {
    Activity {
        source,
        source_id: start.to_string(),
        app: "Brave".into(),
        title: String::new(),
        start_at: DateTime::from_timestamp(start, 0).unwrap(),
        end_at: DateTime::from_timestamp(start + length, 0).unwrap(),
        raw: serde_json::Value::Null,
        extras: Extras {
            idle_secs: Some(no_input),
            ..Extras::default()
        },
    }
}

fn fixture() -> Vec<Activity> {
    vec![
        row(Source::Clockify, 0, 5 * HOUR, 10_800.0),
        row(Source::Clockify, 6 * HOUR, HOUR / 2, 1200.0),
        row(Source::Clockify, DAY, 2 * HOUR, 0.0),
        row(Source::Toggl, DAY, 9 * HOUR, 0.0),
    ]
}

fn by_minutes(effects: &[ThresholdEffect], minutes: i64) -> ThresholdEffect {
    *effects
        .iter()
        .find(|c| c.away_after_secs == minutes * 60)
        .unwrap()
}

#[test]
fn has_one_row_per_choice_lowest_first() {
    let secs: Vec<_> = preview(&fixture())
        .iter()
        .map(|c| c.away_after_secs)
        .collect();
    assert_eq!(secs, [0, 60, 180, 300, 600, 900, 1800, 3600]);
}

#[test]
fn counts_only_hour_long_stretches_at_the_current_threshold() {
    let sixty = by_minutes(&preview(&fixture()), 60);
    // 3 h of Away over 2 days; 7.5 h of Clockify less that is Active.
    assert_eq!(sixty.away_secs_per_day, 3 * HOUR / 2);
    assert_eq!(sixty.active_secs_per_day, (15 * HOUR / 2 - 3 * HOUR) / 2);
}

#[test]
fn a_lower_threshold_adds_the_shorter_stretches() {
    let fifteen = by_minutes(&preview(&fixture()), 15);
    assert_eq!(fifteen.away_secs_per_day, 6000);
}

#[test]
fn rows_split_inside_one_absence_count_as_one_stretch() {
    let split = [
        row(Source::Clockify, 0, 3000, 3000.0),
        row(Source::Clockify, 3000, 3000, 6000.0),
        row(Source::Clockify, 6000, 3000, 9000.0),
    ];
    assert_eq!(by_minutes(&preview(&split), 60).away_secs_per_day, 9000);
}

#[test]
fn ignores_other_sources_and_handles_no_activity() {
    let toggl_only = [row(Source::Toggl, 0, 5 * HOUR, 0.0)];
    for activities in [&toggl_only[..], &[]] {
        assert!(
            preview(activities)
                .iter()
                .all(|c| c.away_secs_per_day == 0 && c.active_secs_per_day == 0)
        );
    }
}
