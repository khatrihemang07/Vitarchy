//! Time Entries for the Day lane, clipped to the Day.

use super::model::LaneEntry;
use crate::{report::entry::WindowEntry, timeline::TimeRange};

/// The entries as lane items, cut to `range` (the Day's UTC range); entries wholly outside it
/// are dropped.
#[must_use]
pub fn lane(entries: &[WindowEntry], range: TimeRange) -> Vec<LaneEntry> {
    entries
        .iter()
        .filter(|e| e.start < range.to && e.stop > range.from)
        .map(|e| LaneEntry {
            start_at: e.start.max(range.from),
            stop_at: e.stop.min(range.to),
            description: e.description.clone(),
            project: e.project_name.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;
    use crate::report::entry::fixtures;

    fn t(text: &str) -> DateTime<Utc> {
        text.parse().unwrap()
    }

    #[test]
    fn entries_are_cut_to_the_day_and_outside_ones_dropped() {
        let mut inside = fixtures::entry(2, "2026-10-17T09:00:00Z", 1, "Inside");
        inside.project_name = Some("Office".into());
        let entries = [
            fixtures::entry(1, "2026-10-16T23:00:00Z", 2, "Late"),
            inside,
            fixtures::entry(3, "2026-10-18T09:00:00Z", 1, "Next day"),
        ];
        let range = (t("2026-10-17T00:00:00Z"), t("2026-10-18T00:00:00Z"));
        let lane = lane(&entries, range.into());
        let shape: Vec<_> = lane
            .iter()
            .map(|e| {
                (
                    e.description.as_str(),
                    e.start_at,
                    e.stop_at,
                    e.project.as_deref(),
                )
            })
            .collect();
        assert_eq!(
            shape,
            [
                (
                    "Late",
                    t("2026-10-17T00:00:00Z"),
                    t("2026-10-17T01:00:00Z"),
                    None
                ),
                (
                    "Inside",
                    t("2026-10-17T09:00:00Z"),
                    t("2026-10-17T10:00:00Z"),
                    Some("Office")
                ),
            ]
        );
    }
}
