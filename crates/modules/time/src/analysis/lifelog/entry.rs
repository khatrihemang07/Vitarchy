//! One stored Time Entry, prepared for analysis.

use chrono::{DateTime, Utc};

/// What an entry with no project is called.
pub const NO_PROJECT: &str = "No project";
/// What an entry with no description is called.
pub const NO_DESCRIPTION: &str = "No description";

/// The form two names are compared in: trimmed and case-folded. The one place that decides
/// when two descriptions or project names count as the same.
#[must_use]
pub fn fold(text: &str) -> String {
    text.trim().to_lowercase()
}

/// A Time Entry with its project named and its description trimmed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifeEntry {
    /// The project's name, or [`NO_PROJECT`].
    pub project: String,
    /// The project's Toggl colour as stored (`#rrggbb`), when it has one.
    pub color: Option<String>,
    /// The description, trimmed; empty when none was written.
    pub description: String,
    /// [`fold`] of the project's name, computed once so matching never allocates.
    pub project_key: String,
    /// [`fold`] of the description as shown ([`NO_DESCRIPTION`] when empty), computed once.
    pub description_key: String,
    /// When it began.
    pub start: DateTime<Utc>,
    /// When it ended.
    pub end: DateTime<Utc>,
}

impl LifeEntry {
    /// Prepares an entry as stored.
    #[must_use]
    pub fn new(
        project: Option<&str>,
        color: Option<&str>,
        description: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Self {
        let mut entry = Self {
            project: project.map_or_else(|| NO_PROJECT.to_owned(), str::to_owned),
            color: color.map(str::to_owned),
            description: description.trim().to_owned(),
            project_key: String::new(),
            description_key: String::new(),
            start,
            end,
        };
        entry.project_key = fold(&entry.project);
        entry.description_key = fold(entry.description_label());
        entry
    }

    /// The description to show: [`NO_DESCRIPTION`] when empty.
    #[must_use]
    pub fn description_label(&self) -> &str {
        if self.description.is_empty() {
            NO_DESCRIPTION
        } else {
            &self.description
        }
    }
}

/// Whole seconds from milliseconds, rounded once per sum.
pub(super) fn seconds(millis: i64) -> i64 {
    (millis + 500) / 1000
}

/// Test helper: an entry on 2026-05 UTC from instants and names.
#[cfg(test)]
pub(super) fn entry(project: &str, description: &str, start: &str, end: &str) -> LifeEntry {
    let project = (project != "-").then_some(project);
    LifeEntry::new(
        project,
        None,
        description,
        start.parse().unwrap(),
        end.parse().unwrap(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_project_and_description_get_plain_names() {
        let e = entry("-", "  ", "2026-05-01T09:00:00Z", "2026-05-01T10:00:00Z");
        assert_eq!(
            (e.project.as_str(), e.description_label()),
            (NO_PROJECT, NO_DESCRIPTION)
        );
        assert_eq!(e.description, "");
    }
}
