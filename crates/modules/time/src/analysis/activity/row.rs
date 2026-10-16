//! One stored Activity row, prepared for analysis.

use chrono::{DateTime, Utc};

use crate::analysis::context::{clean_app, derive_context, trim_invisible};
use crate::analysis::day_split::DayPortion;

/// An Activity row with its app cleaned and its Context derived once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityRow {
    /// The app, without invisible marks.
    pub app: String,
    /// What the app was showing (may be empty).
    pub title: String,
    /// The Context derived from the app and title.
    pub context: String,
    /// When it began.
    pub start: DateTime<Utc>,
    /// When it ended.
    pub end: DateTime<Utc>,
}

impl ActivityRow {
    /// Prepares a row as stored.
    #[must_use]
    pub fn new(app: &str, title: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        let app = clean_app(app);
        Self {
            context: derive_context(&app, title),
            title: trim_invisible(title).to_owned(),
            app,
            start,
            end,
        }
    }
}

/// Length of a portion in milliseconds.
pub(super) fn millis(portion: &DayPortion) -> i64 {
    (portion.end - portion.start).num_milliseconds()
}

/// Test helper: a row from `HH:MM` clock times on 2026-05-01 UTC.
#[cfg(test)]
pub(super) fn at(text: &str) -> DateTime<Utc> {
    text.parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_is_trimmed_of_invisible_marks_the_same_way_its_context_is() {
        let row = ActivityRow::new(
            "Code",
            "\u{200E} main.rs — vitarchy\u{200B}",
            at("2026-05-01T09:00:00Z"),
            at("2026-05-01T09:05:00Z"),
        );
        assert_eq!(row.title, "main.rs — vitarchy");
        assert_eq!(row.context, "vitarchy");
    }
}
