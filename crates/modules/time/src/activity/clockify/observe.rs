//! Clockify's view of its Activity for the Timeline, with its no-input time turned into Away time.

use crate::{
    activity::model::Activity,
    timeline::{
        away_rules::AwayRules,
        model::Observation,
        no_input::{NoInputRow, away_tails},
    },
};

/// Clockify's `idle_secs` are the no-input time at the end of a row, carried on across the rows
/// it splits, so the batch is judged by [`away_tails`] (one decision per no-input stretch). Each
/// row becomes an active part followed by an Away one where its tail is Away. Zero-length parts
/// are dropped.
#[must_use]
pub fn observe(activities: &[&Activity], rules: &AwayRules) -> Vec<Observation> {
    let rows: Vec<NoInputRow> = activities.iter().map(|a| NoInputRow::of(a)).collect();
    activities
        .iter()
        .zip(away_tails(&rows, rules))
        .flat_map(|(activity, away)| parts(activity, away))
        .collect()
}

fn parts(activity: &Activity, away: chrono::Duration) -> impl Iterator<Item = Observation> {
    let away_from = activity.end_at - away;
    let part = |start_at, end_at, away| Observation {
        source: activity.source,
        start_at,
        end_at,
        app: activity.app.clone(),
        title: activity.title.clone(),
        url: activity.extras.url.clone(),
        away: Some(away),
    };
    [
        part(activity.start_at, away_from, false),
        part(away_from, activity.end_at, true),
    ]
    .into_iter()
    .filter(|o| o.end_at > o.start_at)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::{model::Extras, source::Source};
    use chrono::DateTime;

    const HOUR: i64 = 3600;

    /// A Brave row from second `start` for `length` seconds, with `idle_secs` of no input.
    fn row_at(start: i64, length: i64, idle_secs: f64) -> Activity {
        Activity {
            source: Source::Clockify,
            source_id: start.to_string(),
            app: "Brave".into(),
            title: "t".into(),
            start_at: DateTime::from_timestamp(start, 0).unwrap(),
            end_at: DateTime::from_timestamp(start + length, 0).unwrap(),
            raw: serde_json::json!({}),
            extras: Extras {
                url: Some("https://example.com".into()),
                idle_secs: Some(idle_secs),
                ..Extras::default()
            },
        }
    }

    fn row(length: i64, idle_secs: f64) -> Activity {
        row_at(100, length, idle_secs)
    }

    fn observed(activities: &[Activity]) -> Vec<Observation> {
        let refs: Vec<&Activity> = activities.iter().collect();
        observe(&refs, &AwayRules::current())
    }

    fn shape(activities: &[Activity]) -> Vec<(i64, i64, Option<bool>)> {
        observed(activities)
            .iter()
            .map(|o| (o.start_at.timestamp(), o.end_at.timestamp(), o.away))
            .collect()
    }

    #[test]
    fn no_idle_time_is_one_active_observation() {
        assert_eq!(shape(&[row(100, 0.0)]), [(100, 200, Some(false))]);
    }

    #[test]
    fn a_pause_under_an_hour_stays_active() {
        let end = 100 + 2 * HOUR;
        assert_eq!(shape(&[row(2 * HOUR, 3599.0)]), [(100, end, Some(false))]);
    }

    #[test]
    fn an_hour_of_no_input_becomes_a_trailing_away_part() {
        let end = 100 + 2 * HOUR;
        assert_eq!(
            shape(&[row(2 * HOUR, 3600.0)]),
            [
                (100, end - HOUR, Some(false)),
                (end - HOUR, end, Some(true))
            ]
        );
    }

    #[test]
    fn no_input_longer_than_the_row_is_capped_at_the_row() {
        assert_eq!(
            shape(&[row(2 * HOUR, 50_000.0)]),
            [(100, 100 + 2 * HOUR, Some(true))]
        );
    }

    #[test]
    fn rows_split_inside_one_absence_are_all_away() {
        let third = 50 * 60;
        let rows = [
            row_at(0, third, 3000.0),
            row_at(third, third, 6000.0),
            row_at(2 * third, third, 9000.0),
        ];
        assert_eq!(
            shape(&rows),
            [
                (0, third, Some(true)),
                (third, 2 * third, Some(true)),
                (2 * third, 3 * third, Some(true))
            ]
        );
    }

    #[test]
    fn every_part_keeps_the_app_title_and_url() {
        let parts = observed(&[row(2 * HOUR, 3600.0)]);
        assert_eq!(parts.len(), 2);
        for o in parts {
            assert_eq!(o.app, "Brave");
            assert_eq!(o.url.as_deref(), Some("https://example.com"));
        }
    }
}
