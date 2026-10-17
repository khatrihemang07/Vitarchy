//! What a Report groups by. Each dimension is a row of data: how it keys an entry.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::entry::WindowEntry;

/// A way to group Time Entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Grouping {
    /// By description, ignoring case and surrounding spaces.
    Description,
    /// By project.
    Project,
    /// By tag; an entry with several tags counts under each.
    Tag,
}

/// One group an entry belongs to: a stable key and the label seen for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keyed {
    /// Identity of the group; empty for the "nothing to group by" group.
    pub key: String,
    /// How this entry spells the group.
    pub label: String,
}

struct Dimension {
    grouping: Grouping,
    keys: fn(&WindowEntry) -> Vec<Keyed>,
}

fn keyed(key: impl Into<String>, label: impl Into<String>) -> Keyed {
    Keyed {
        key: key.into(),
        label: label.into(),
    }
}

fn by_description(entry: &WindowEntry) -> Vec<Keyed> {
    let text = entry.description.trim();
    if text.is_empty() {
        return vec![keyed("", "(no description)")];
    }
    vec![keyed(text.to_lowercase(), text)]
}

fn by_project(entry: &WindowEntry) -> Vec<Keyed> {
    match entry.project_id {
        None => vec![keyed("", "(no project)")],
        Some(id) => {
            let name = entry.project_name.as_deref().unwrap_or("(unknown project)");
            vec![keyed(id.to_string(), name)]
        }
    }
}

fn by_tag(entry: &WindowEntry) -> Vec<Keyed> {
    let mut tags: Vec<&String> = entry.tags.iter().collect();
    tags.sort();
    tags.dedup();
    if tags.is_empty() {
        return vec![keyed("", "(no tag)")];
    }
    tags.into_iter()
        .map(|t| keyed(t.as_str(), t.as_str()))
        .collect()
}

static DIMENSIONS: [Dimension; 3] = [
    Dimension {
        grouping: Grouping::Description,
        keys: by_description,
    },
    Dimension {
        grouping: Grouping::Project,
        keys: by_project,
    },
    Dimension {
        grouping: Grouping::Tag,
        keys: by_tag,
    },
];

impl Grouping {
    /// The groups the entry belongs to (one, except for tags).
    #[must_use]
    pub fn keys(self, entry: &WindowEntry) -> Vec<Keyed> {
        DIMENSIONS
            .iter()
            .find(|d| d.grouping == self)
            .map_or_else(Vec::new, |d| (d.keys)(entry))
    }
}

/// The most common label; ties go to the alphabetically first so output is stable.
#[must_use]
pub fn most_common_label<'a>(labels: impl Iterator<Item = &'a str>) -> String {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for label in labels {
        *counts.entry(label).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))
        .map_or_else(String::new, |(label, _)| label.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::entry::fixtures::entry;

    fn description_keys(text: &str) -> Vec<Keyed> {
        Grouping::Description.keys(&entry(1, "2026-10-17T09:00:00Z", 1, text))
    }

    #[test]
    fn descriptions_match_ignoring_case_and_surrounding_spaces() {
        assert_eq!(
            description_keys("Office")[0].key,
            description_keys("office ")[0].key
        );
        assert_eq!(description_keys(" Office ")[0].label, "Office");
    }

    #[test]
    fn nothing_to_group_by_gets_a_placeholder_group() {
        assert_eq!(description_keys("  ")[0].label, "(no description)");
        let bare = entry(1, "2026-10-17T09:00:00Z", 1, "x");
        assert_eq!(Grouping::Project.keys(&bare)[0].label, "(no project)");
        assert_eq!(Grouping::Tag.keys(&bare)[0].label, "(no tag)");
    }

    #[test]
    fn a_project_is_keyed_by_id_and_labelled_by_name() {
        let mut e = entry(1, "2026-10-17T09:00:00Z", 1, "x");
        e.project_id = Some(30);
        e.project_name = Some("Office".to_owned());
        assert_eq!(Grouping::Project.keys(&e), [keyed("30", "Office")]);
    }

    #[test]
    fn an_entry_with_two_tags_is_in_two_groups() {
        let mut e = entry(1, "2026-10-17T09:00:00Z", 1, "x");
        e.tags = vec!["b".into(), "a".into(), "a".into()];
        let keys: Vec<String> = Grouping::Tag.keys(&e).into_iter().map(|k| k.key).collect();
        assert_eq!(keys, ["a", "b"]);
    }

    #[test]
    fn the_label_is_the_most_common_spelling() {
        let label = most_common_label(["office", "Office", "Office"].into_iter());
        assert_eq!(label, "Office");
        assert_eq!(most_common_label(["b", "a"].into_iter()), "a");
    }
}
