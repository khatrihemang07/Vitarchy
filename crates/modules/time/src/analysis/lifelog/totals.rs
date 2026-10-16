//! Where the logged time went: per project (with its descriptions) and per description.
//!
//! Every figure counts each entry's own time, so overlapping entries add up to more
//! than the clock; `logged_seconds` is that same sum, which keeps every share consistent.

use std::collections::{BTreeSet, HashMap};

use chrono::NaiveDate;
use serde::Serialize;
use utoipa::ToSchema;

use super::LifeEntry;
use super::entry::{millis, seconds};
use crate::analysis::{Period, split_by_day};

/// Time on one description.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct DescriptionTotal {
    /// The description, or "No description".
    pub description: String,
    /// Seconds.
    pub seconds: i64,
    /// Entries that touch the Period.
    pub entries: i64,
    /// Days with at least one such entry.
    pub days: i64,
}

/// Time on one project.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct ProjectTotal {
    /// The project's name, or "No project".
    pub name: String,
    /// The project's Toggl colour (`#rrggbb`), when it has one.
    pub color: Option<String>,
    /// Seconds.
    pub seconds: i64,
    /// Entries that touch the Period.
    pub entries: i64,
    /// Days with at least one such entry.
    pub days: i64,
    /// Its descriptions, longest first.
    pub descriptions: Vec<DescriptionTotal>,
}

/// Logged time over a Period.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct LifeLogTotals {
    /// Seconds across all projects (each entry's own time).
    pub logged_seconds: i64,
    /// Projects, longest first.
    pub by_project: Vec<ProjectTotal>,
    /// Descriptions across all projects, longest first.
    pub by_description: Vec<DescriptionTotal>,
}

#[derive(Default)]
struct Tally {
    millis: i64,
    entries: i64,
    days: BTreeSet<NaiveDate>,
}

impl Tally {
    fn add(&mut self, millis: i64, dates: &[NaiveDate]) {
        self.millis += millis;
        self.entries += 1;
        self.days.extend(dates);
    }

    fn into_total(self, description: String) -> DescriptionTotal {
        DescriptionTotal {
            description,
            seconds: seconds(self.millis),
            entries: self.entries,
            days: i64::try_from(self.days.len()).unwrap_or(i64::MAX),
        }
    }
}

#[derive(Default)]
struct ProjectTally {
    color: Option<String>,
    own: Tally,
    descriptions: HashMap<String, Tally>,
}

/// Longest first; names break ties so output is stable.
fn longest_first(items: &mut [DescriptionTotal]) {
    items.sort_by(|a, b| {
        b.seconds
            .cmp(&a.seconds)
            .then_with(|| a.description.cmp(&b.description))
    });
}

/// Totals of the entries' portions inside the Period.
#[must_use]
pub fn totals(period: &Period, entries: &[LifeEntry]) -> LifeLogTotals {
    let mut projects: HashMap<&str, ProjectTally> = HashMap::new();
    let mut descriptions: HashMap<&str, Tally> = HashMap::new();
    for entry in entries {
        let portions = split_by_day(period, entry.start, entry.end);
        if portions.is_empty() {
            continue;
        }
        let spent: i64 = portions.iter().map(millis).sum();
        let dates: Vec<NaiveDate> = portions.iter().map(|p| p.date).collect();
        let project = projects.entry(&entry.project).or_default();
        if project.color.is_none() {
            project.color.clone_from(&entry.color);
        }
        project.own.add(spent, &dates);
        project
            .descriptions
            .entry(entry.description_label().to_owned())
            .or_default()
            .add(spent, &dates);
        descriptions
            .entry(entry.description_label())
            .or_default()
            .add(spent, &dates);
    }
    let mut by_description: Vec<DescriptionTotal> = descriptions
        .into_iter()
        .map(|(name, tally)| tally.into_total(name.to_owned()))
        .collect();
    longest_first(&mut by_description);
    let mut by_project: Vec<ProjectTotal> = projects
        .into_iter()
        .map(|(name, tally)| {
            let mut descriptions: Vec<DescriptionTotal> = tally
                .descriptions
                .into_iter()
                .map(|(d, t)| t.into_total(d))
                .collect();
            longest_first(&mut descriptions);
            let own = tally.own.into_total(String::new());
            ProjectTotal {
                name: name.to_owned(),
                color: tally.color,
                seconds: own.seconds,
                entries: own.entries,
                days: own.days,
                descriptions,
            }
        })
        .collect();
    by_project.sort_by(|a, b| b.seconds.cmp(&a.seconds).then_with(|| a.name.cmp(&b.name)));
    LifeLogTotals {
        logged_seconds: by_project.iter().map(|p| p.seconds).sum(),
        by_project,
        by_description,
    }
}

#[cfg(test)]
mod tests {
    use super::super::entry::entry;
    use super::*;

    const HOUR: i64 = 3600;

    fn run(rows: &[(&str, &str, &str, &str)]) -> LifeLogTotals {
        let period = Period::parse("2026-05-01", "2026-05-03", "UTC").unwrap();
        let entries: Vec<LifeEntry> = rows.iter().map(|(p, d, s, e)| entry(p, d, s, e)).collect();
        totals(&period, &entries)
    }

    #[test]
    fn projects_rank_by_time_with_their_descriptions_nested() {
        let t = run(&[
            (
                "office",
                "inside_office",
                "2026-05-01T09:00:00Z",
                "2026-05-01T17:00:00Z",
            ),
            (
                "office",
                "travel to_office",
                "2026-05-01T08:00:00Z",
                "2026-05-01T09:00:00Z",
            ),
            (
                "body",
                "workout",
                "2026-05-01T18:00:00Z",
                "2026-05-01T19:00:00Z",
            ),
        ]);
        assert_eq!(t.logged_seconds, 10 * HOUR);
        assert_eq!(t.by_project[0].name, "office");
        assert_eq!(t.by_project[0].seconds, 9 * HOUR);
        let nested: Vec<&str> = t.by_project[0]
            .descriptions
            .iter()
            .map(|d| d.description.as_str())
            .collect();
        assert_eq!(nested, ["inside_office", "travel to_office"]);
        assert_eq!(t.by_project[1].name, "body");
    }

    #[test]
    fn entries_and_days_are_counted_once_each() {
        let t = run(&[
            (
                "office",
                "job",
                "2026-05-01T09:00:00Z",
                "2026-05-01T10:00:00Z",
            ),
            (
                "office",
                "job",
                "2026-05-01T11:00:00Z",
                "2026-05-01T12:00:00Z",
            ),
            (
                "office",
                "job",
                "2026-05-02T09:00:00Z",
                "2026-05-02T10:00:00Z",
            ),
        ]);
        let job = &t.by_description[0];
        assert_eq!((job.entries, job.days, job.seconds), (3, 2, 3 * HOUR));
        assert_eq!((t.by_project[0].entries, t.by_project[0].days), (3, 2));
    }

    #[test]
    fn a_midnight_crossing_entry_is_one_entry_on_two_days() {
        let t = run(&[(
            "sleep",
            "sleep",
            "2026-05-01T23:00:00Z",
            "2026-05-02T07:00:00Z",
        )]);
        assert_eq!((t.by_project[0].entries, t.by_project[0].days), (1, 2));
        assert_eq!(t.by_project[0].seconds, 8 * HOUR);
    }

    #[test]
    fn only_the_part_inside_the_period_counts() {
        let t = run(&[(
            "sleep",
            "sleep",
            "2026-05-03T22:00:00Z",
            "2026-05-04T06:00:00Z",
        )]);
        assert_eq!(t.logged_seconds, 2 * HOUR);
    }

    #[test]
    fn overlapping_entries_each_count_their_own_time() {
        let t = run(&[
            (
                "office",
                "job",
                "2026-05-01T09:00:00Z",
                "2026-05-01T11:00:00Z",
            ),
            (
                "body",
                "workout",
                "2026-05-01T10:00:00Z",
                "2026-05-01T11:00:00Z",
            ),
        ]);
        assert_eq!(t.logged_seconds, 3 * HOUR);
    }

    #[test]
    fn a_description_used_in_two_projects_is_one_row_across_projects() {
        let t = run(&[
            (
                "office",
                "remote",
                "2026-05-01T09:00:00Z",
                "2026-05-01T10:00:00Z",
            ),
            (
                "work",
                "remote",
                "2026-05-02T09:00:00Z",
                "2026-05-02T10:00:00Z",
            ),
        ]);
        assert_eq!(t.by_description.len(), 1);
        assert_eq!(
            (t.by_description[0].entries, t.by_description[0].days),
            (2, 2)
        );
    }

    #[test]
    fn missing_project_and_description_have_plain_names_and_the_colour_is_kept() {
        let period = Period::parse("2026-05-01", "2026-05-01", "UTC").unwrap();
        let none = entry("-", "", "2026-05-01T09:00:00Z", "2026-05-01T10:00:00Z");
        let mut coloured = entry(
            "office",
            "job",
            "2026-05-01T10:00:00Z",
            "2026-05-01T11:00:00Z",
        );
        coloured.color = Some("#2a78d6".to_owned());
        let t = totals(&period, &[none, coloured]);
        let names: Vec<&str> = t.by_project.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["No project", "office"]);
        assert_eq!(t.by_project[1].color.as_deref(), Some("#2a78d6"));
        assert!(
            t.by_description
                .iter()
                .any(|d| d.description == "No description")
        );
    }

    #[test]
    fn nothing_logged_is_an_empty_total() {
        let t = run(&[]);
        assert_eq!(t.logged_seconds, 0);
        assert!(t.by_project.is_empty() && t.by_description.is_empty());
    }
}
