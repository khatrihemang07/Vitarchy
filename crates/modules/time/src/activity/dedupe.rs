//! Turns parsed rows into distinct Activity, the same way for every Source.

use std::collections::HashMap;

use super::model::{Activity, Parsed};

/// Collects parsed rows (`None` is an unusable row). A repeated `source_id` keeps the row
/// that ends latest, because an ongoing Activity is written again as it grows; the
/// others count as skipped.
#[must_use]
pub fn dedupe_latest_end(rows: impl IntoIterator<Item = Option<Activity>>) -> Parsed {
    let mut position: HashMap<String, usize> = HashMap::new();
    let mut activities: Vec<Activity> = Vec::new();
    let mut skipped = 0;
    for activity in rows {
        let Some(activity) = activity else {
            skipped += 1;
            continue;
        };
        if let Some(&at) = position.get(&activity.source_id) {
            skipped += 1;
            if activity.end_at > activities[at].end_at {
                activities[at] = activity;
            }
        } else {
            position.insert(activity.source_id.clone(), activities.len());
            activities.push(activity);
        }
    }
    Parsed {
        activities,
        skipped,
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use serde_json::json;

    use super::*;
    use crate::activity::{model::Extras, source::Source};

    fn activity(id: &str, end_secs: i64) -> Activity {
        Activity {
            source: Source::Toggl,
            source_id: id.into(),
            app: String::new(),
            title: String::new(),
            start_at: DateTime::from_timestamp(0, 0).unwrap(),
            end_at: DateTime::from_timestamp(end_secs, 0).unwrap(),
            raw: json!({}),
            extras: Extras::default(),
        }
    }

    #[test]
    fn distinct_rows_are_kept_in_order_and_unusable_ones_are_counted() {
        let parsed = dedupe_latest_end([Some(activity("a", 1)), None, Some(activity("b", 1))]);
        let ids: Vec<_> = parsed
            .activities
            .iter()
            .map(|a| a.source_id.as_str())
            .collect();
        assert_eq!((ids, parsed.skipped), (vec!["a", "b"], 1));
    }

    #[test]
    fn a_repeated_id_keeps_the_later_end_wherever_it_appears() {
        let later_first = dedupe_latest_end([Some(activity("a", 9)), Some(activity("a", 2))]);
        let later_last = dedupe_latest_end([Some(activity("a", 2)), Some(activity("a", 9))]);
        for parsed in [later_first, later_last] {
            assert_eq!(parsed.activities.len(), 1);
            assert_eq!(parsed.activities[0].end_at.timestamp(), 9);
            assert_eq!(parsed.skipped, 1);
        }
    }
}
