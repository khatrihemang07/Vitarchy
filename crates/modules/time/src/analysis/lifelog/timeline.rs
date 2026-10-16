//! One Day as a lane of entries, each clipped to the Day. Overlapping entries stay separate.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::LifeEntry;
use crate::analysis::{Period, split_by_day};

/// One entry's part of the Day.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct LifeLogSpan {
    /// Start (inclusive), clipped to the Day.
    pub start: DateTime<Utc>,
    /// End (exclusive), clipped to the Day.
    pub end: DateTime<Utc>,
    /// The project's name, or "No project".
    pub project: String,
    /// The project's Toggl colour (`#rrggbb`), when it has one.
    pub color: Option<String>,
    /// The description, or "No description".
    pub description: String,
}

/// The spans inside the Period's first Day, earliest first.
#[must_use]
pub fn timeline(period: &Period, entries: &[LifeEntry]) -> Vec<LifeLogSpan> {
    let mut spans: Vec<LifeLogSpan> = entries
        .iter()
        .flat_map(|entry| {
            split_by_day(period, entry.start, entry.end)
                .into_iter()
                .map(|portion| LifeLogSpan {
                    start: portion.start,
                    end: portion.end,
                    project: entry.project.clone(),
                    color: entry.color.clone(),
                    description: entry.description_label().to_owned(),
                })
        })
        .collect();
    spans.sort_by_key(|span| (span.start, span.end));
    spans
}

#[cfg(test)]
mod tests {
    use super::super::entry::entry;
    use super::*;

    fn lane(
        tz: &str,
        day: &str,
        rows: &[(&str, &str, &str, &str)],
    ) -> Vec<(String, String, String, String)> {
        let period = Period::parse(day, day, tz).unwrap();
        let entries: Vec<LifeEntry> = rows.iter().map(|(p, d, s, e)| entry(p, d, s, e)).collect();
        timeline(&period, &entries)
            .into_iter()
            .map(|s| {
                (
                    s.description,
                    s.project,
                    s.start.format("%H:%M").to_string(),
                    s.end.format("%H:%M").to_string(),
                )
            })
            .collect()
    }

    fn row(
        description: &str,
        project: &str,
        start: &str,
        end: &str,
    ) -> (String, String, String, String) {
        (
            description.to_owned(),
            project.to_owned(),
            start.to_owned(),
            end.to_owned(),
        )
    }

    #[test]
    fn entries_are_in_time_order_with_project_and_description() {
        let spans = lane(
            "UTC",
            "2026-05-01",
            &[
                (
                    "body",
                    "workout",
                    "2026-05-01T18:00:00Z",
                    "2026-05-01T19:00:00Z",
                ),
                (
                    "office",
                    "job",
                    "2026-05-01T09:00:00Z",
                    "2026-05-01T17:00:00Z",
                ),
            ],
        );
        assert_eq!(
            spans,
            [
                row("job", "office", "09:00", "17:00"),
                row("workout", "body", "18:00", "19:00")
            ]
        );
    }

    #[test]
    fn sleep_is_clipped_to_each_day_it_touches() {
        let rows = [(
            "sleep",
            "sleep",
            "2026-05-01T23:00:00Z",
            "2026-05-02T07:00:00Z",
        )];
        assert_eq!(
            lane("UTC", "2026-05-01", &rows),
            [row("sleep", "sleep", "23:00", "00:00")]
        );
        assert_eq!(
            lane("UTC", "2026-05-02", &rows),
            [row("sleep", "sleep", "00:00", "07:00")]
        );
    }

    #[test]
    fn overlapping_entries_are_both_kept() {
        let spans = lane(
            "UTC",
            "2026-05-01",
            &[
                (
                    "office",
                    "job",
                    "2026-05-01T09:00:00Z",
                    "2026-05-01T12:00:00Z",
                ),
                (
                    "body",
                    "workout",
                    "2026-05-01T11:00:00Z",
                    "2026-05-01T12:30:00Z",
                ),
            ],
        );
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn a_day_without_entries_has_no_spans_and_a_blank_description_is_named() {
        assert!(
            lane(
                "UTC",
                "2026-05-01",
                &[("p", "x", "2026-04-30T09:00:00Z", "2026-04-30T10:00:00Z")]
            )
            .is_empty()
        );
        let spans = lane(
            "UTC",
            "2026-05-01",
            &[("-", "", "2026-05-01T09:00:00Z", "2026-05-01T10:00:00Z")],
        );
        assert_eq!(
            spans,
            [row("No description", "No project", "09:00", "10:00")]
        );
    }
}
