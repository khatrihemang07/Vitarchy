//! One Step of a Routine: a label and what it is matched to.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::analysis::lifelog::LifeEntry;

/// What a Step is matched against.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MatchKind {
    /// The entry's description.
    Description,
    /// The entry's project, by name.
    Project,
}

impl MatchKind {
    /// The text stored in the database and used by the API.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Description => "description",
            Self::Project => "project",
        }
    }

    /// The kind stored as `text`, or `None` for anything else.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "description" => Some(Self::Description),
            "project" => Some(Self::Project),
            _ => None,
        }
    }
}

/// One part of a Routine.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct Step {
    /// What the person calls this Step.
    pub label: String,
    /// Whether `value` is a description or a project name.
    pub kind: MatchKind,
    /// The description or project name to match: exact after trimming, ignoring case.
    pub value: String,
}

/// Equality for matching: trimmed and case-insensitive.
fn same(a: &str, b: &str) -> bool {
    a.trim().to_lowercase() == b.trim().to_lowercase()
}

impl Step {
    /// Whether `entry` belongs to this Step. A project is matched by name (Toggl projects have
    /// no other stable handle in the analysis rows; renaming one in Toggl ends the match).
    #[must_use]
    pub fn matches(&self, entry: &LifeEntry) -> bool {
        match self.kind {
            MatchKind::Description => same(&entry.description, &self.value),
            MatchKind::Project => same(&entry.project, &self.value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::entry;
    use super::*;

    fn step(kind: MatchKind, value: &str) -> Step {
        Step {
            label: "x".into(),
            kind,
            value: value.into(),
        }
    }

    #[test]
    fn descriptions_match_exactly_after_trimming_ignoring_case() {
        let e = entry(
            "office",
            " Inside_Office ",
            "2026-05-01T09:00:00Z",
            "2026-05-01T10:00:00Z",
        );
        assert!(step(MatchKind::Description, "inside_office ").matches(&e));
        assert!(!step(MatchKind::Description, "inside").matches(&e));
        assert!(!step(MatchKind::Description, "office").matches(&e));
    }

    #[test]
    fn projects_match_by_name_and_a_missing_project_is_no_project() {
        let e = entry(
            "Office",
            "x",
            "2026-05-01T09:00:00Z",
            "2026-05-01T10:00:00Z",
        );
        assert!(step(MatchKind::Project, "office").matches(&e));
        let none = entry("-", "x", "2026-05-01T09:00:00Z", "2026-05-01T10:00:00Z");
        assert!(step(MatchKind::Project, "No project").matches(&none));
    }

    #[test]
    fn kinds_round_trip_their_stored_text() {
        for kind in [MatchKind::Description, MatchKind::Project] {
            assert_eq!(MatchKind::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(MatchKind::parse("tag"), None);
    }
}
