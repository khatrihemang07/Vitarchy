//! Which Time Entries a Report includes.

use std::collections::BTreeSet;

use super::entry::WindowEntry;

/// Choices combine with OR inside one set and AND across sets; an empty set allows everything.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filter {
    /// Project ids.
    pub projects: BTreeSet<i64>,
    /// Tag names.
    pub tags: BTreeSet<String>,
    /// Client ids, reached through the entry's project.
    pub clients: BTreeSet<i64>,
    /// Case-insensitive text the description must contain.
    pub search: Option<String>,
}

fn allows<T: Ord>(set: &BTreeSet<T>, wanted: impl Fn(&T) -> bool) -> bool {
    set.is_empty() || set.iter().any(wanted)
}

impl Filter {
    /// Whether the entry passes every constraint.
    #[must_use]
    pub fn matches(&self, entry: &WindowEntry) -> bool {
        allows(&self.projects, |id| entry.project_id == Some(*id))
            && allows(&self.clients, |id| entry.client_id == Some(*id))
            && allows(&self.tags, |tag| entry.tags.contains(tag))
            && self.search.as_ref().is_none_or(|text| {
                entry
                    .description
                    .to_lowercase()
                    .contains(&text.to_lowercase())
            })
    }

    /// How many constraints are set, for logs (never the values).
    #[must_use]
    pub fn counts(&self) -> [usize; 4] {
        [
            self.projects.len(),
            self.tags.len(),
            self.clients.len(),
            usize::from(self.search.is_some()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::entry::fixtures::entry;

    fn tagged(project: Option<i64>, client: Option<i64>, tags: &[&str], d: &str) -> WindowEntry {
        let mut e = entry(1, "2026-10-17T09:00:00Z", 1, d);
        e.project_id = project;
        e.client_id = client;
        e.tags = tags.iter().map(|t| (*t).to_owned()).collect();
        e
    }

    #[test]
    fn an_empty_filter_allows_everything() {
        assert!(Filter::default().matches(&tagged(None, None, &[], "")));
    }

    #[test]
    fn values_within_one_set_are_alternatives() {
        let f = Filter {
            projects: [1, 2].into(),
            ..Filter::default()
        };
        assert!(f.matches(&tagged(Some(2), None, &[], "")));
        assert!(!f.matches(&tagged(Some(3), None, &[], "")));
        assert!(!f.matches(&tagged(None, None, &[], "")));
    }

    #[test]
    fn different_sets_must_all_match() {
        let f = Filter {
            projects: [1].into(),
            tags: ["focus".to_owned()].into(),
            ..Filter::default()
        };
        assert!(f.matches(&tagged(Some(1), None, &["focus", "x"], "")));
        assert!(!f.matches(&tagged(Some(1), None, &["x"], "")));
        assert!(!f.matches(&tagged(Some(2), None, &["focus"], "")));
    }

    #[test]
    fn a_client_matches_through_the_project() {
        let f = Filter {
            clients: [9].into(),
            ..Filter::default()
        };
        assert!(f.matches(&tagged(Some(1), Some(9), &[], "")));
        assert!(!f.matches(&tagged(Some(1), None, &[], "")));
    }

    #[test]
    fn the_search_ignores_case() {
        let f = Filter {
            search: Some("OFF".to_owned()),
            ..Filter::default()
        };
        assert!(f.matches(&tagged(None, None, &[], "Commute to office")));
        assert!(!f.matches(&tagged(None, None, &[], "Gym")));
    }
}
