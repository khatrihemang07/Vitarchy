//! Trims a Source's observations that overlap each other, so each Source says one thing at a time.

use std::collections::HashMap;

use super::model::Observation;
use crate::activity::Source;

/// For each Source separately, makes a later observation begin where the earlier one ends,
/// dropping any that end up empty. Sources are never trimmed against each other.
#[must_use]
pub fn trim_self_overlaps(mut observations: Vec<Observation>) -> Vec<Observation> {
    observations.sort_by_key(|o| (o.start_at, o.end_at));
    let mut reached: HashMap<Source, chrono::DateTime<chrono::Utc>> = HashMap::new();
    let mut kept = Vec::with_capacity(observations.len());
    for mut observation in observations {
        if let Some(&end) = reached.get(&observation.source) {
            observation.start_at = observation.start_at.max(end);
        }
        if observation.end_at <= observation.start_at {
            continue;
        }
        reached.insert(observation.source, observation.end_at);
        kept.push(observation);
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::test_support::{at, obs};

    #[test]
    fn a_later_row_starts_where_the_earlier_one_ends() {
        let rows = vec![
            obs(Source::Toggl, 0, 10, "A", None),
            obs(Source::Toggl, 7, 20, "B", None),
        ];
        let trimmed = trim_self_overlaps(rows);
        assert_eq!((trimmed[1].start_at, trimmed[1].end_at), (at(10), at(20)));
        assert_eq!(trimmed[0].end_at, at(10));
    }

    #[test]
    fn a_row_inside_an_earlier_one_disappears() {
        let rows = vec![
            obs(Source::Toggl, 0, 100, "A", None),
            obs(Source::Toggl, 10, 20, "B", None),
        ];
        assert_eq!(trim_self_overlaps(rows).len(), 1);
    }

    #[test]
    fn different_sources_keep_their_overlap() {
        let rows = vec![
            obs(Source::Toggl, 0, 10, "A", None),
            obs(Source::Clockify, 5, 15, "A", Some(false)),
        ];
        let trimmed = trim_self_overlaps(rows);
        assert_eq!(trimmed.len(), 2);
        assert_eq!(trimmed[1].start_at, at(5));
    }

    #[test]
    fn touching_rows_and_unsorted_input_are_left_alone() {
        let rows = vec![
            obs(Source::Toggl, 10, 20, "B", None),
            obs(Source::Toggl, 0, 10, "A", None),
        ];
        let trimmed = trim_self_overlaps(rows);
        assert_eq!(trimmed.len(), 2);
        assert_eq!(trimmed[0].app, "A");
        assert_eq!(trimmed[1].start_at, at(10));
    }
}
