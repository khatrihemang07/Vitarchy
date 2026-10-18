//! Tests for `no_input`.

use chrono::{DateTime, Duration};

use super::away_rules::AwayRules;
use super::no_input::*;

const MIN: i64 = 60;

fn row(start_min: i64, length_min: i64, no_input_min: i64, watching: bool) -> NoInputRow {
    NoInputRow {
        start_at: DateTime::from_timestamp(1_000_000 + start_min * MIN, 0).unwrap(),
        end_at: DateTime::from_timestamp(1_000_000 + (start_min + length_min) * MIN, 0).unwrap(),
        no_input_secs: f64::from(i32::try_from(no_input_min * MIN).unwrap()),
        watching,
    }
}

fn away_minutes(rows: &[NoInputRow]) -> Vec<i64> {
    away_tails(rows, &AwayRules::current())
        .iter()
        .map(Duration::num_minutes)
        .collect()
}

#[test]
fn three_continuing_rows_of_fifty_minutes_are_one_hundred_fifty_minutes_away() {
    let rows = [
        row(0, 50, 50, false),
        row(50, 50, 100, false),
        row(100, 50, 150, false),
    ];
    assert_eq!(away_minutes(&rows), [50, 50, 50]);
}

#[test]
fn a_row_each_would_be_under_the_threshold_alone() {
    assert_eq!(away_minutes(&[row(0, 50, 50, false)]), [0]);
}

#[test]
fn a_neighbour_whose_no_input_reset_is_a_separate_stretch() {
    // Merged, 50 + 10 min would be an hour of Away; apart, neither is.
    assert_eq!(
        away_minutes(&[row(0, 50, 50, false), row(50, 50, 10, false)]),
        [0, 0]
    );
    // Apart, the later row is Away on its own and the earlier one is not.
    assert_eq!(
        away_minutes(&[row(0, 50, 50, false), row(50, 80, 80, false)]),
        [0, 80]
    );
}

#[test]
fn away_starts_inside_the_row_where_the_hour_is_reached() {
    // Input stops 30 min before the first row ends; 90 min without input in all.
    let rows = [row(0, 60, 30, false), row(60, 60, 90, false)];
    assert_eq!(away_minutes(&rows), [30, 60]);
    let rows = [row(0, 60, 60, false), row(60, 60, 120, false)];
    assert_eq!(away_minutes(&rows), [60, 60]);
}

#[test]
fn a_gap_between_rows_ends_the_stretch() {
    let rows = [row(0, 70, 70, false), row(71, 70, 140, false)];
    assert_eq!(away_minutes(&rows), [70, 70]);
    let rows = [row(0, 40, 40, false), row(41, 40, 80, false)];
    assert_eq!(away_minutes(&rows), [0, 0]);
}

#[test]
fn a_few_seconds_either_way_still_continues() {
    let mut second = row(50, 50, 100, false);
    second.start_at += Duration::seconds(2);
    second.end_at += Duration::seconds(2);
    assert_eq!(away_minutes(&[row(0, 50, 50, false), second]), [50, 50]);
}

#[test]
fn a_watching_stretch_split_at_a_title_change_gets_one_cap_in_total() {
    // 3 h 30 of YouTube in two rows: 2 h stays Active once, so 1 h 30 is Away.
    let rows = [row(0, 105, 105, true), row(105, 105, 210, true)];
    assert_eq!(away_minutes(&rows), [0, 90]);
}

#[test]
fn a_watching_stretch_under_cap_plus_threshold_is_not_away() {
    let rows = [row(0, 75, 75, true), row(75, 75, 150, true)];
    assert_eq!(away_minutes(&rows), [0, 0]);
}

#[test]
fn only_the_watching_part_of_a_mixed_stretch_is_kept_active() {
    // 1 h in a document, then 2 h of video: 2 h kept, the remaining hour is Away.
    let rows = [row(0, 60, 60, false), row(60, 120, 180, true)];
    assert_eq!(away_minutes(&rows), [0, 60]);
}

#[test]
fn a_stretch_split_at_midnight_is_one_stretch() {
    // 22:00 to 02:30 without input, split at midnight into 2 h and 2 h 30.
    let rows = [row(0, 120, 120, false), row(120, 150, 270, false)];
    assert_eq!(away_minutes(&rows), [120, 150]);
}

#[test]
fn results_follow_the_input_order_even_when_it_is_not_in_time_order() {
    let rows = [
        row(50, 50, 100, false),
        row(0, 50, 50, false),
        row(100, 50, 150, false),
    ];
    assert_eq!(away_minutes(&rows), [50, 50, 50]);
}

#[test]
fn no_input_before_the_first_row_is_capped_at_the_row() {
    // The stretch began before the first stored row; only the stored part counts.
    assert_eq!(away_minutes(&[row(0, 90, 500, false)]), [90]);
}

#[test]
fn no_rows_give_no_tails() {
    assert!(away_minutes(&[]).is_empty());
}

mod reach {
    use super::super::model::TimeRange;
    use super::*;
    use crate::activity::{Activity, Extras, Source};

    fn activity(source: Source, start_min: i64, length_min: i64, no_input_min: i64) -> Activity {
        let r = row(start_min, length_min, no_input_min, false);
        Activity {
            source,
            source_id: start_min.to_string(),
            app: "Brave".into(),
            title: String::new(),
            start_at: r.start_at,
            end_at: r.end_at,
            raw: serde_json::Value::Null,
            extras: Extras {
                idle_secs: Some(r.no_input_secs),
                ..Extras::default()
            },
        }
    }

    fn span(from_min: i64, to_min: i64) -> TimeRange {
        TimeRange {
            from: row(from_min, 1, 0, false).start_at,
            to: row(to_min, 1, 0, false).start_at,
        }
    }

    #[test]
    fn a_row_carrying_on_into_the_range_is_taken_in() {
        let near = [
            activity(Source::Clockify, 0, 50, 50),
            activity(Source::Clockify, 50, 50, 100),
        ];
        assert_eq!(
            super::super::no_input::reach(span(50, 100), &near),
            span(0, 100)
        );
    }

    #[test]
    fn a_row_carrying_on_out_of_the_range_is_taken_in() {
        let near = [
            activity(Source::Clockify, 0, 50, 50),
            activity(Source::Clockify, 50, 50, 100),
        ];
        assert_eq!(
            super::super::no_input::reach(span(0, 50), &near),
            span(0, 100)
        );
    }

    #[test]
    fn a_touching_row_whose_no_input_reset_is_left_out() {
        let near = [
            activity(Source::Clockify, 0, 50, 50),
            activity(Source::Clockify, 50, 50, 5),
        ];
        assert_eq!(
            super::super::no_input::reach(span(50, 100), &near),
            span(50, 100)
        );
    }

    #[test]
    fn other_sources_are_not_pulled_in() {
        let near = [
            activity(Source::Toggl, 0, 50, 50),
            activity(Source::Clockify, 50, 50, 100),
        ];
        assert_eq!(
            super::super::no_input::reach(span(50, 100), &near),
            span(50, 100)
        );
    }
}
