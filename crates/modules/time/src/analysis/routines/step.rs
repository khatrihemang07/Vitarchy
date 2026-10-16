//! One Step of a Routine: a label and what it is matched to.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::analysis::lifelog::{LifeEntry, fold};

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

impl Step {
    /// This Step ready to test many entries: its value is folded once, not per entry.
    #[must_use]
    pub fn matcher(&self) -> StepMatcher {
        StepMatcher {
            kind: self.kind,
            key: fold(&self.value),
        }
    }
}

/// A [`Step`] prepared for matching. A project is matched by name (Toggl projects have no other
/// stable handle in the analysis rows; renaming one in Toggl ends the match). A description is
/// matched as shown, so "No description" matches the entries that have none, just as
/// "No project" matches the entries without a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepMatcher {
    kind: MatchKind,
    key: String,
}

impl StepMatcher {
    /// Whether `entry` belongs to the Step. Allocation-free: entries carry their folded names.
    #[must_use]
    pub fn matches(&self, entry: &LifeEntry) -> bool {
        match self.kind {
            MatchKind::Description => entry.description_key == self.key,
            MatchKind::Project => entry.project_key == self.key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::entry;
    use super::*;
    use crate::analysis::lifelog::NO_DESCRIPTION;

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
        assert!(
            step(MatchKind::Description, "inside_office ")
                .matcher()
                .matches(&e)
        );
        assert!(!step(MatchKind::Description, "inside").matcher().matches(&e));
        assert!(!step(MatchKind::Description, "office").matcher().matches(&e));
    }

    #[test]
    fn no_description_matches_entries_that_have_none() {
        let blank = entry("p", "  ", "2026-05-01T09:00:00Z", "2026-05-01T10:00:00Z");
        let worded = entry("p", "x", "2026-05-01T09:00:00Z", "2026-05-01T10:00:00Z");
        let matcher = step(MatchKind::Description, NO_DESCRIPTION).matcher();
        assert!(matcher.matches(&blank));
        assert!(!matcher.matches(&worded));
    }

    #[test]
    fn projects_match_by_name_and_a_missing_project_is_no_project() {
        let e = entry(
            "Office",
            "x",
            "2026-05-01T09:00:00Z",
            "2026-05-01T10:00:00Z",
        );
        assert!(step(MatchKind::Project, "office").matcher().matches(&e));
        let none = entry("-", "x", "2026-05-01T09:00:00Z", "2026-05-01T10:00:00Z");
        assert!(
            step(MatchKind::Project, "No project")
                .matcher()
                .matches(&none)
        );
    }

    #[test]
    fn kinds_round_trip_their_stored_text() {
        for kind in [MatchKind::Description, MatchKind::Project] {
            assert_eq!(MatchKind::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(MatchKind::parse("tag"), None);
    }
}
