//! Series for saved Routines: each Step's start, end and length, and the whole Routine's length.

use super::measure::{Span, series_of};
use crate::analysis::{
    Period,
    insights::{Measure, Point, Series, Subject, SubjectKind},
    lifelog::LifeEntry,
    routines::{Routine, occurrences},
};

/// Series for every Routine over the reach.
pub(super) fn series(reach: &Period, routines: &[Routine], entries: &[LifeEntry]) -> Vec<Series> {
    let tz = reach.tz();
    let mut all = Vec::new();
    for routine in routines {
        let found = occurrences(reach, &routine.steps, entries);
        for (index, step) in routine.steps.iter().enumerate() {
            let spans: Vec<Span> = found
                .iter()
                .filter_map(|o| {
                    let span = o.steps[index].as_ref()?;
                    Some(Span {
                        date: o.date,
                        start: span.start,
                        end: span.end,
                    })
                })
                .collect();
            let subject = Subject {
                kind: SubjectKind::RoutineStep,
                name: step.label.clone(),
            };
            all.extend(series_of(&subject, tz, &spans));
        }
        let points: Vec<Point> = found
            .iter()
            .filter_map(|o| {
                Some(Point {
                    date: o.date,
                    value: minutes_of(o.door_to_door_seconds?),
                })
            })
            .collect();
        if !points.is_empty() {
            let subject = Subject {
                kind: SubjectKind::RoutineTotal,
                name: routine.name.clone(),
            };
            all.push(Series {
                subject,
                measure: Measure::Length,
                points,
            });
        }
    }
    all
}

// Door-to-door times are far inside f64's exact range.
#[allow(clippy::cast_precision_loss)]
fn minutes_of(seconds: i64) -> f64 {
    seconds as f64 / 60.0
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;
    use crate::analysis::insights::{Measure, SubjectKind};
    use crate::analysis::routines::{MatchKind, Step};

    fn step(label: &str, value: &str) -> Step {
        Step {
            label: label.into(),
            kind: MatchKind::Description,
            value: value.into(),
        }
    }

    fn routine(steps: Vec<Step>) -> Routine {
        let at: DateTime<Utc> = "2026-05-01T00:00:00Z".parse().unwrap();
        Routine {
            id: 1,
            name: "Office day".into(),
            steps,
            created_at: at,
            updated_at: at,
        }
    }

    fn entry(description: &str, start: &str, end: &str) -> LifeEntry {
        LifeEntry::new(
            Some("p"),
            None,
            description,
            start.parse().unwrap(),
            end.parse().unwrap(),
        )
    }

    fn reach() -> Period {
        Period::parse("2026-04-06", "2026-05-31", "UTC").unwrap()
    }

    /// `to` 08:00-08:30 and `in` 08:45-16:45 on May `day`.
    fn day(day: u32) -> Vec<LifeEntry> {
        vec![
            entry(
                "to",
                &format!("2026-05-{day:02}T08:00:00Z"),
                &format!("2026-05-{day:02}T08:30:00Z"),
            ),
            entry(
                "in",
                &format!("2026-05-{day:02}T08:45:00Z"),
                &format!("2026-05-{day:02}T16:45:00Z"),
            ),
        ]
    }

    fn find<'a>(all: &'a [Series], kind: SubjectKind, name: &str, measure: Measure) -> &'a Series {
        all.iter()
            .find(|s| s.subject.kind == kind && s.subject.name == name && s.measure == measure)
            .expect("series exists")
    }

    fn values(series: &Series) -> Vec<f64> {
        series.points.iter().map(|p| p.value).collect()
    }

    #[test]
    fn each_step_is_measured_by_start_end_and_length_on_every_day() {
        let entries: Vec<_> = [4, 5, 6].into_iter().flat_map(day).collect();
        let all = series(
            &reach(),
            &[routine(vec![step("Commute", "to"), step("Office", "in")])],
            &entries,
        );
        assert_eq!(
            values(find(
                &all,
                SubjectKind::RoutineStep,
                "Commute",
                Measure::Start
            )),
            [480.0; 3]
        );
        assert_eq!(
            values(find(
                &all,
                SubjectKind::RoutineStep,
                "Commute",
                Measure::End
            )),
            [510.0; 3]
        );
        assert_eq!(
            values(find(
                &all,
                SubjectKind::RoutineStep,
                "Office",
                Measure::Length
            )),
            [480.0; 3]
        );
        assert_eq!(
            find(&all, SubjectKind::RoutineStep, "Office", Measure::Start).points[0]
                .date
                .to_string(),
            "2026-05-04"
        );
    }

    #[test]
    fn the_whole_routine_is_a_length_series_over_complete_days_only() {
        let mut entries: Vec<_> = [4, 5].into_iter().flat_map(day).collect();
        entries.push(entry("in", "2026-05-06T09:00:00Z", "2026-05-06T17:00:00Z"));
        let all = series(
            &reach(),
            &[routine(vec![step("Commute", "to"), step("Office", "in")])],
            &entries,
        );
        let total = find(
            &all,
            SubjectKind::RoutineTotal,
            "Office day",
            Measure::Length,
        );
        assert_eq!(values(total), [525.0, 525.0]);
        assert_eq!(
            find(&all, SubjectKind::RoutineStep, "Office", Measure::Length)
                .points
                .len(),
            3
        );
    }

    #[test]
    fn no_routines_have_no_series() {
        assert!(series(&reach(), &[], &day(4)).is_empty());
    }
}
