//! Finds a Routine's Occurrence on each Day.
//!
//! A Day's entries (those that **start** in it) are taken in start order. Each Step, in turn,
//! claims the first matching entry after the previous Step's entries, plus the entries right after
//! it that also match (a trip logged in two pieces is one Step). A Step with no match is missing
//! and the search for the next Step carries on from the same place, so a Day without the first
//! trip still shows the rest. Days where no Step happened are not Occurrences.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::Step;
use crate::analysis::Period;
use crate::analysis::lifelog::LifeEntry;

/// When one Step happened on a Day.
#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Eq)]
pub struct StepSpan {
    /// Start of the first matching entry.
    pub start: DateTime<Utc>,
    /// End of the last consecutive matching entry.
    pub end: DateTime<Utc>,
    /// Seconds from `start` to `end`.
    pub seconds: i64,
}

/// One Day on which a Routine happened.
#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Eq)]
pub struct Occurrence {
    /// Local date.
    #[schema(value_type = String, format = Date)]
    pub date: NaiveDate,
    /// True when every Step happened.
    pub complete: bool,
    /// One per Step in order; `null` for a Step that did not happen.
    pub steps: Vec<Option<StepSpan>>,
    /// Seconds between each Step's end and the next Step's start (one fewer than Steps);
    /// `null` when either Step is missing. Never negative.
    pub gaps: Vec<Option<i64>>,
    /// First Step's start to last Step's end; only for a complete Occurrence.
    pub door_to_door_seconds: Option<i64>,
}

/// The Routine's Occurrences in the Period, earliest Day first.
#[must_use]
pub fn occurrences(period: &Period, steps: &[Step], entries: &[LifeEntry]) -> Vec<Occurrence> {
    let mut days: BTreeMap<NaiveDate, Vec<&LifeEntry>> = BTreeMap::new();
    for entry in entries {
        if entry.start >= period.start() && entry.start < period.end() {
            let date = entry.start.with_timezone(&period.tz()).date_naive();
            days.entry(date).or_default().push(entry);
        }
    }
    days.into_iter()
        .filter_map(|(date, mut day)| {
            day.sort_by_key(|e| e.start);
            let spans = match_steps(steps, &day);
            spans
                .iter()
                .any(Option::is_some)
                .then(|| build(date, spans))
        })
        .collect()
}

/// Each Step's span in one Day's entries (in start order), or `None` when it did not happen.
fn match_steps(steps: &[Step], day: &[&LifeEntry]) -> Vec<Option<StepSpan>> {
    let mut cursor = 0;
    steps
        .iter()
        .map(|step| {
            let first = cursor + day[cursor..].iter().position(|e| step.matches(e))?;
            let run = day[first..].iter().take_while(|e| step.matches(e)).count();
            cursor = first + run;
            let end = day[first..cursor].iter().map(|e| e.end).max()?;
            let start = day[first].start;
            Some(StepSpan {
                start,
                end,
                seconds: (end - start).num_seconds(),
            })
        })
        .collect()
}

fn build(date: NaiveDate, spans: Vec<Option<StepSpan>>) -> Occurrence {
    let gaps = spans
        .windows(2)
        .map(|pair| match pair {
            [Some(before), Some(after)] => Some((after.start - before.end).num_seconds().max(0)),
            _ => None,
        })
        .collect();
    let complete = spans.iter().all(Option::is_some);
    let door_to_door_seconds = match (spans.first(), spans.last()) {
        (Some(Some(first)), Some(Some(last))) if complete => {
            Some((last.end - first.start).num_seconds())
        }
        _ => None,
    };
    Occurrence {
        date,
        complete,
        steps: spans,
        gaps,
        door_to_door_seconds,
    }
}

#[cfg(test)]
mod tests {
    use super::super::{MatchKind, entry};
    use super::*;

    fn period() -> Period {
        Period::parse("2026-05-01", "2026-05-07", "UTC").unwrap()
    }

    fn step(value: &str) -> Step {
        Step {
            label: value.into(),
            kind: MatchKind::Description,
            value: value.into(),
        }
    }

    fn office_steps() -> Vec<Step> {
        ["to", "in", "from"].map(step).to_vec()
    }

    fn e(description: &str, day: u32, start: &str, end: &str) -> LifeEntry {
        entry(
            "p",
            description,
            &format!("2026-05-{day:02}T{start}:00Z"),
            &format!("2026-05-{day:02}T{end}:00Z"),
        )
    }

    fn full_day(day: u32) -> Vec<LifeEntry> {
        vec![
            e("to", day, "08:00", "09:00"),
            e("in", day, "09:15", "17:00"),
            e("from", day, "17:10", "18:00"),
        ]
    }

    #[test]
    fn a_full_day_is_complete_with_step_times_gaps_and_door_to_door() {
        let found = occurrences(&period(), &office_steps(), &full_day(4));
        assert_eq!(found.len(), 1);
        let day = &found[0];
        assert!(day.complete);
        assert_eq!(day.date.to_string(), "2026-05-04");
        let seconds: Vec<i64> = day
            .steps
            .iter()
            .map(|s| s.as_ref().unwrap().seconds)
            .collect();
        assert_eq!(seconds, [3600, 27900, 3000]);
        assert_eq!(day.gaps, [Some(900), Some(600)]);
        assert_eq!(day.door_to_door_seconds, Some(10 * 3600));
    }

    #[test]
    fn a_missing_step_makes_the_day_incomplete_and_leaves_the_rest_measured() {
        let entries = vec![e("in", 4, "09:15", "17:00"), e("from", 4, "17:10", "18:00")];
        let day = &occurrences(&period(), &office_steps(), &entries)[0];
        assert!(!day.complete);
        assert!(day.steps[0].is_none() && day.steps[1].is_some() && day.steps[2].is_some());
        assert_eq!(day.gaps, [None, Some(600)]);
        assert_eq!(day.door_to_door_seconds, None);
    }

    #[test]
    fn days_with_no_step_are_not_occurrences() {
        let entries = vec![e("other", 4, "09:00", "10:00")];
        assert!(occurrences(&period(), &office_steps(), &entries).is_empty());
    }

    #[test]
    fn consecutive_matching_entries_form_one_step() {
        let entries = vec![
            e("to", 4, "08:00", "08:30"),
            e("to", 4, "08:35", "09:00"),
            e("in", 4, "09:15", "17:00"),
            e("from", 4, "17:10", "18:00"),
        ];
        let day = &occurrences(&period(), &office_steps(), &entries)[0];
        let first = day.steps[0].as_ref().unwrap();
        assert_eq!(first.seconds, 3600);
    }

    #[test]
    fn steps_are_found_in_order_and_an_early_later_step_does_not_count() {
        // "from" before "in": "from" cannot follow "in", so it is missing.
        let entries = vec![
            e("to", 4, "07:00", "07:30"),
            e("from", 4, "08:00", "08:30"),
            e("in", 4, "09:00", "17:00"),
        ];
        let day = &occurrences(&period(), &office_steps(), &entries)[0];
        assert!(day.steps[0].is_some() && day.steps[1].is_some() && day.steps[2].is_none());
    }

    #[test]
    fn only_the_first_run_of_a_step_counts() {
        let entries = vec![
            e("to", 4, "08:00", "09:00"),
            e("x", 4, "09:00", "09:30"),
            e("to", 4, "10:00", "11:00"),
            e("in", 4, "11:00", "12:00"),
        ];
        let day = &occurrences(&period(), &office_steps(), &entries)[0];
        assert_eq!(day.steps[0].as_ref().unwrap().seconds, 3600);
    }

    #[test]
    fn an_entry_belongs_to_the_day_it_starts_in() {
        // Night shift ending after midnight: the "in" entry starts on day 4.
        let entries = vec![
            e("to", 4, "20:00", "21:00"),
            entry("p", "in", "2026-05-04T21:15:00Z", "2026-05-05T05:00:00Z"),
            e("from", 5, "05:10", "06:00"),
        ];
        let found = occurrences(&period(), &office_steps(), &entries);
        assert_eq!(found.len(), 2);
        assert!(!found[0].complete && found[0].steps[2].is_none());
        assert!(found[1].steps[0].is_none() && found[1].steps[2].is_some());
    }

    #[test]
    fn project_steps_match_by_project_name() {
        let steps = vec![Step {
            label: "Office".into(),
            kind: MatchKind::Project,
            value: "office".into(),
        }];
        let entries = vec![entry(
            "Office",
            "anything",
            "2026-05-04T09:00:00Z",
            "2026-05-04T10:00:00Z",
        )];
        assert_eq!(occurrences(&period(), &steps, &entries).len(), 1);
    }

    #[test]
    fn overlapping_steps_never_give_a_negative_gap() {
        let entries = vec![e("to", 4, "08:00", "09:00"), e("in", 4, "08:50", "17:00")];
        let day = &occurrences(&period(), &office_steps(), &entries)[0];
        assert_eq!(day.gaps[0], Some(0));
    }

    #[test]
    fn days_come_earliest_first() {
        let mut entries = full_day(6);
        entries.extend(full_day(2));
        let dates: Vec<String> = occurrences(&period(), &office_steps(), &entries)
            .iter()
            .map(|o| o.date.to_string())
            .collect();
        assert_eq!(dates, ["2026-05-02", "2026-05-06"]);
    }
}
