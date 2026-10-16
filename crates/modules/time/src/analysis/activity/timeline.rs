//! One Day as a lane of spans: consecutive rows in the same Context merged.

use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::ActivityRow;
use crate::analysis::{Period, split_by_day};

/// Rows of the same Context closer together than this are one span.
pub const MERGE_GAP_SECS: i64 = 60;

/// An unbroken stretch in one Context.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct Span {
    /// Start (inclusive), clipped to the Day.
    pub start: DateTime<Utc>,
    /// End (exclusive), clipped to the Day.
    pub end: DateTime<Utc>,
    /// The app.
    pub app: String,
    /// The Context within the app.
    pub context: String,
}

/// The spans inside the Period's first Day, earliest first.
#[must_use]
pub fn timeline(period: &Period, rows: &[ActivityRow]) -> Vec<Span> {
    let mut pieces: Vec<Span> = rows
        .iter()
        .flat_map(|row| {
            split_by_day(period, row.start, row.end)
                .into_iter()
                .map(|portion| Span {
                    start: portion.start,
                    end: portion.end,
                    app: row.app.clone(),
                    context: row.context.clone(),
                })
        })
        .collect();
    pieces.sort_by_key(|span| span.start);
    let gap = Duration::seconds(MERGE_GAP_SECS);
    let mut merged: Vec<Span> = Vec::with_capacity(pieces.len());
    for piece in pieces {
        match merged.last_mut() {
            Some(last)
                if last.app == piece.app
                    && last.context == piece.context
                    && piece.start - last.end <= gap =>
            {
                last.end = last.end.max(piece.end);
            }
            _ => merged.push(piece),
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::super::row::at;
    use super::*;

    fn lane(
        tz: &str,
        day: &str,
        rows: &[(&str, &str, &str, &str)],
    ) -> Vec<(String, String, String)> {
        let period = Period::parse(day, day, tz).unwrap();
        let rows: Vec<ActivityRow> = rows
            .iter()
            .map(|(app, title, s, e)| ActivityRow::new(app, title, at(s), at(e)))
            .collect();
        timeline(&period, &rows)
            .into_iter()
            .map(|s| {
                (
                    s.context,
                    s.start.format("%H:%M").to_string(),
                    s.end.format("%H:%M").to_string(),
                )
            })
            .collect()
    }

    fn entry(context: &str, start: &str, end: &str) -> (String, String, String) {
        (context.to_owned(), start.to_owned(), end.to_owned())
    }

    #[test]
    fn rows_of_the_same_context_close_together_merge_into_one_span() {
        let spans = lane(
            "UTC",
            "2026-05-01",
            &[
                (
                    "Code",
                    "a.rs — vitarchy",
                    "2026-05-01T09:00:00Z",
                    "2026-05-01T09:10:00Z",
                ),
                (
                    "Code",
                    "b.rs — vitarchy",
                    "2026-05-01T09:10:30Z",
                    "2026-05-01T09:20:00Z",
                ),
            ],
        );
        assert_eq!(spans, [entry("vitarchy", "09:00", "09:20")]);
    }

    #[test]
    fn a_gap_of_exactly_the_limit_merges_but_one_second_more_does_not() {
        let rows = |second_start| {
            lane(
                "UTC",
                "2026-05-01",
                &[
                    (
                        "Spotify",
                        "",
                        "2026-05-01T09:00:00Z",
                        "2026-05-01T09:10:00Z",
                    ),
                    ("Spotify", "", second_start, "2026-05-01T09:20:00Z"),
                ],
            )
        };
        assert_eq!(rows("2026-05-01T09:11:00Z").len(), 1);
        assert_eq!(rows("2026-05-01T09:11:01Z").len(), 2);
    }

    #[test]
    fn a_different_context_in_between_splits_a_context() {
        let spans = lane(
            "UTC",
            "2026-05-01",
            &[
                (
                    "Code",
                    "a — p",
                    "2026-05-01T09:00:00Z",
                    "2026-05-01T09:10:00Z",
                ),
                (
                    "Spotify",
                    "",
                    "2026-05-01T09:10:00Z",
                    "2026-05-01T09:12:00Z",
                ),
                (
                    "Code",
                    "a — p",
                    "2026-05-01T09:12:00Z",
                    "2026-05-01T09:30:00Z",
                ),
            ],
        );
        assert_eq!(spans.len(), 3);
    }

    #[test]
    fn the_same_context_name_in_two_apps_stays_apart() {
        let spans = lane(
            "UTC",
            "2026-05-01",
            &[
                (
                    "Code",
                    "x — shared",
                    "2026-05-01T09:00:00Z",
                    "2026-05-01T09:10:00Z",
                ),
                (
                    "Cursor",
                    "x — shared",
                    "2026-05-01T09:10:00Z",
                    "2026-05-01T09:20:00Z",
                ),
            ],
        );
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn rows_are_clipped_to_the_day() {
        let spans = lane(
            "UTC",
            "2026-05-02",
            &[(
                "Spotify",
                "",
                "2026-05-01T20:00:00Z",
                "2026-05-02T03:00:00Z",
            )],
        );
        assert_eq!(spans, [entry("Spotify", "00:00", "03:00")]);
    }

    #[test]
    fn spans_come_out_in_time_order_whatever_the_row_order() {
        let spans = lane(
            "UTC",
            "2026-05-01",
            &[
                ("B", "", "2026-05-01T12:00:00Z", "2026-05-01T12:30:00Z"),
                ("A", "", "2026-05-01T08:00:00Z", "2026-05-01T08:30:00Z"),
            ],
        );
        assert_eq!(spans[0].0, "A");
    }

    #[test]
    fn a_day_with_no_activity_has_no_spans() {
        assert!(
            lane(
                "UTC",
                "2026-05-01",
                &[("A", "", "2026-04-30T08:00:00Z", "2026-04-30T09:00:00Z")]
            )
            .is_empty()
        );
    }
}
