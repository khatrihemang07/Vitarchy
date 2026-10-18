//! Clockify's view of an Activity for the Timeline, with its idle seconds turned into Away time.

use chrono::Duration;

use crate::{activity::model::Activity, timeline::model::Observation};

/// Clockify's idle seconds are cumulative over the row and can exceed it, so the away time
/// is taken as the trailing `min(idle_secs, length)`: an active observation followed by an
/// Away one. Zero-length parts are dropped.
#[must_use]
pub fn observe(activity: &Activity) -> Vec<Observation> {
    let length = activity.end_at - activity.start_at;
    let idle = idle_duration(activity.extras.idle_secs.unwrap_or(0.0));
    let away_from = activity.end_at - idle.min(length);
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
    .collect()
}

/// Idle seconds as a duration; negative or absurdly large values are bounded first so the
/// conversion cannot overflow (the caller caps it at the row's length anyway).
#[allow(clippy::cast_possible_truncation)]
fn idle_duration(secs: f64) -> Duration {
    Duration::milliseconds((secs.clamp(0.0, 1.0e9) * 1000.0).round() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::{model::Extras, source::Source};
    use chrono::DateTime;

    fn row(idle_secs: f64) -> Activity {
        Activity {
            source: Source::Clockify,
            source_id: "a".into(),
            app: "Brave".into(),
            title: "t".into(),
            start_at: DateTime::from_timestamp(100, 0).unwrap(),
            end_at: DateTime::from_timestamp(200, 0).unwrap(),
            raw: serde_json::json!({}),
            extras: Extras {
                url: Some("https://example.com".into()),
                idle_secs: Some(idle_secs),
                ..Extras::default()
            },
        }
    }

    fn shape(activity: &Activity) -> Vec<(i64, i64, Option<bool>)> {
        observe(activity)
            .iter()
            .map(|o| (o.start_at.timestamp(), o.end_at.timestamp(), o.away))
            .collect()
    }

    #[test]
    fn no_idle_time_is_one_active_observation() {
        assert_eq!(shape(&row(0.0)), [(100, 200, Some(false))]);
    }

    #[test]
    fn idle_time_becomes_a_trailing_away_part() {
        assert_eq!(
            shape(&row(30.0)),
            [(100, 170, Some(false)), (170, 200, Some(true))]
        );
    }

    #[test]
    fn idle_longer_than_the_row_is_capped_at_the_row() {
        assert_eq!(shape(&row(5000.0)), [(100, 200, Some(true))]);
    }

    #[test]
    fn every_part_keeps_the_app_title_and_url() {
        for o in observe(&row(30.0)) {
            assert_eq!(o.app, "Brave");
            assert_eq!(o.url.as_deref(), Some("https://example.com"));
        }
    }
}
