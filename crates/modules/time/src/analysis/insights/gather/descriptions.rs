//! Series for frequent descriptions that no Routine Step already covers.

use std::collections::HashSet;

use super::measure::{Span, series_of};
use crate::analysis::{
    Period,
    insights::{Series, Subject, SubjectKind},
    lifelog::{LifeEntry, description_stats, fold},
};

/// Most descriptions measured, the most frequent first.
pub(super) const MAX_DESCRIPTIONS: usize = 15;

/// Series for each description that is a habit in `report`, measured over `reach`. `covered`
/// holds the folded descriptions a Routine Step already reports on.
pub(super) fn series(
    report: &Period,
    reach: &Period,
    entries: &[LifeEntry],
    covered: &HashSet<String>,
) -> Vec<Series> {
    let tz = reach.tz();
    description_stats(report, entries)
        .into_iter()
        .filter(|stat| !covered.contains(&fold(&stat.description)))
        .take(MAX_DESCRIPTIONS)
        .flat_map(|stat| {
            let spans: Vec<Span> = entries
                .iter()
                .filter(|e| {
                    e.description == stat.description
                        && e.start >= reach.start()
                        && e.start < reach.end()
                })
                .map(|e| Span {
                    date: e.start.with_timezone(&tz).date_naive(),
                    start: e.start,
                    end: e.end,
                })
                .collect();
            let subject = Subject {
                kind: SubjectKind::Description,
                name: stat.description,
            };
            series_of(&subject, tz, &spans)
        })
        .collect()
}

#[cfg(test)]
// The expected figures are exact by construction (small whole minutes).
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::analysis::insights::{Measure, SubjectKind};

    fn entry(description: &str, day: u32) -> LifeEntry {
        LifeEntry::new(
            Some("p"),
            None,
            description,
            format!("2026-05-{day:02}T18:30:00Z").parse().unwrap(),
            format!("2026-05-{day:02}T19:10:00Z").parse().unwrap(),
        )
    }

    fn period(from: &str) -> Period {
        Period::parse(from, "2026-05-31", "UTC").unwrap()
    }

    #[test]
    fn a_habit_is_measured_by_start_end_and_length() {
        let entries: Vec<_> = (10..16).map(|d| entry("workout", d)).collect();
        let all = series(
            &period("2026-05-01"),
            &period("2026-05-01"),
            &entries,
            &HashSet::new(),
        );
        assert_eq!(all.len(), 3);
        assert!(
            all.iter()
                .all(|s| s.subject.kind == SubjectKind::Description && s.subject.name == "workout")
        );
        let length = all.iter().find(|s| s.measure == Measure::Length).unwrap();
        assert_eq!(length.points.len(), 6);
        assert!(length.points.iter().all(|p| p.value == 40.0));
        let start = all.iter().find(|s| s.measure == Measure::Start).unwrap();
        assert!(start.points.iter().all(|p| p.value == 1110.0));
    }

    #[test]
    fn a_description_on_fewer_than_five_days_is_not_a_habit() {
        let entries: Vec<_> = (10..14).map(|d| entry("workout", d)).collect();
        assert!(
            series(
                &period("2026-05-01"),
                &period("2026-05-01"),
                &entries,
                &HashSet::new()
            )
            .is_empty()
        );
    }

    #[test]
    fn a_description_a_routine_step_covers_is_left_out() {
        let entries: Vec<_> = (10..16).map(|d| entry("Workout", d)).collect();
        let covered = HashSet::from([fold("workout")]);
        assert!(
            series(
                &period("2026-05-01"),
                &period("2026-05-01"),
                &entries,
                &covered
            )
            .is_empty()
        );
    }

    #[test]
    fn days_before_the_period_but_inside_the_reach_are_measured_for_change() {
        let mut entries: Vec<_> = (20..26).map(|d| entry("workout", d)).collect();
        entries.extend((5..8).map(|d| entry("workout", d)));
        let all = series(
            &period("2026-05-20"),
            &period("2026-04-06"),
            &entries,
            &HashSet::new(),
        );
        let length = all.iter().find(|s| s.measure == Measure::Length).unwrap();
        assert_eq!(length.points.len(), 9);
    }
}
