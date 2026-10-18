//! Toggl's view of an Activity for the Timeline: exactly what it recorded.

use crate::{
    activity::model::Activity,
    timeline::{away_rules::AwayRules, model::Observation},
};

/// One observation per Activity. Toggl's idle flag is never set, so what it says about
/// Away is "unknown" rather than "not away"; the Away rules do not apply to it.
#[must_use]
pub fn observe(activities: &[&Activity], _rules: &AwayRules) -> Vec<Observation> {
    activities.iter().map(|activity| one(activity)).collect()
}

fn one(activity: &Activity) -> Observation {
    Observation {
        source: activity.source,
        start_at: activity.start_at,
        end_at: activity.end_at,
        app: activity.app.clone(),
        title: activity.title.clone(),
        url: activity.extras.url.clone(),
        away: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::{model::Extras, source::Source};
    use chrono::DateTime;

    #[test]
    fn passes_through_with_away_unknown_even_when_flagged_idle() {
        let activity = Activity {
            source: Source::Toggl,
            source_id: "a".into(),
            app: "Code".into(),
            title: "main.rs".into(),
            start_at: DateTime::from_timestamp(0, 0).unwrap(),
            end_at: DateTime::from_timestamp(60, 0).unwrap(),
            raw: serde_json::json!({}),
            extras: Extras {
                idle: Some(true),
                ..Extras::default()
            },
        };
        let observed = observe(&[&activity], &AwayRules::current());
        assert_eq!(observed.len(), 1);
        assert_eq!((observed[0].away, observed[0].app.as_str()), (None, "Code"));
        assert_eq!(observed[0].end_at, activity.end_at);
    }
}
