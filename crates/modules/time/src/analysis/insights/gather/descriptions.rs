//! Series for frequent descriptions that no Routine Step already covers.

use std::collections::HashSet;

use chrono::NaiveDate;

use super::measure::{Span, one_per_day, series_of};
use crate::analysis::{
    Period,
    insights::{Series, Subject, SubjectKind},
    lifelog::{LifeEntry, attributed_days, description_stats, fold, minute_of_day},
};

/// Most descriptions measured, the most frequent first.
pub(super) const MAX_DESCRIPTIONS: usize = 15;

/// Series for each description that is a habit over `reach`, measured on every Day of it, one
/// point per Day: the first start, the last end and the total length of that Day's entries.
/// Habits are chosen over the whole reach (not just the report), so a short report still gets
/// the changes between its two windows; whether a *report* Day count is enough for a typical
/// time, weekday difference or outlier is decided later, from the report's own points.
/// `covered` holds the folded descriptions a Routine Step already reports on.
pub(super) fn series(
    reach: &Period,
    entries: &[LifeEntry],
    covered: &HashSet<String>,
) -> Vec<Series> {
    let tz = reach.tz();
    description_stats(reach, entries)
        .into_iter()
        .filter(|stat| !covered.contains(&fold(&stat.description)))
        .take(MAX_DESCRIPTIONS)
        .flat_map(|stat| {
            let subject = Subject {
                kind: SubjectKind::Description,
                name: stat.description,
                routine: None,
            };
            let spans = daily_spans(reach, entries, &subject.name);
            series_of(&subject, tz, &spans)
        })
        .collect()
}

/// The Day each entry of `description` in `reach` counts for (a night that straddles midnight
/// is one Day, as in the Life log's habit counts), merged to one span per Day.
fn daily_spans(reach: &Period, entries: &[LifeEntry], description: &str) -> Vec<Span> {
    let tz = reach.tz();
    let found: Vec<&LifeEntry> = entries
        .iter()
        .filter(|e| {
            e.description == description && e.start >= reach.start() && e.start < reach.end()
        })
        .collect();
    let dates: Vec<NaiveDate> = found
        .iter()
        .map(|e| e.start.with_timezone(&tz).date_naive())
        .collect();
    let starts: Vec<f64> = found.iter().map(|e| minute_of_day(tz, e.start)).collect();
    let spans = attributed_days(&dates, &starts)
        .into_iter()
        .zip(found)
        .map(|(date, e)| Span::continuous(date, e.start, e.end))
        .collect();
    one_per_day(spans)
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
        let all = series(&period("2026-05-01"), &entries, &HashSet::new());
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
        assert!(series(&period("2026-05-01"), &entries, &HashSet::new()).is_empty());
    }

    #[test]
    fn a_description_a_routine_step_covers_is_left_out() {
        let entries: Vec<_> = (10..16).map(|d| entry("Workout", d)).collect();
        let covered = HashSet::from([fold("workout")]);
        assert!(series(&period("2026-05-01"), &entries, &covered).is_empty());
    }

    #[test]
    fn days_before_the_period_but_inside_the_reach_are_measured_for_change() {
        let mut entries: Vec<_> = (20..26).map(|d| entry("workout", d)).collect();
        entries.extend((5..8).map(|d| entry("workout", d)));
        let all = series(&period("2026-04-06"), &entries, &HashSet::new());
        let length = all.iter().find(|s| s.measure == Measure::Length).unwrap();
        assert_eq!(length.points.len(), 9);
    }

    /// A description's entries on one Day: `(start, end)` clock text.
    fn day_of(description: &str, day: u32, parts: &[(&str, &str)]) -> Vec<LifeEntry> {
        parts
            .iter()
            .map(|(from, to)| {
                LifeEntry::new(
                    Some("p"),
                    None,
                    description,
                    format!("2026-05-{day:02}T{from}:00Z").parse().unwrap(),
                    format!("2026-05-{day:02}T{to}:00Z").parse().unwrap(),
                )
            })
            .collect()
    }

    fn of(all: &[Series], measure: Measure) -> Vec<(u32, f64)> {
        let mut found: Vec<_> = all
            .iter()
            .find(|s| s.measure == measure)
            .unwrap()
            .points
            .iter()
            .map(|p| (p.date.format("%d").to_string().parse().unwrap(), p.value))
            .collect();
        found.sort_by_key(|(day, _)| *day);
        found
    }

    #[test]
    fn a_description_logged_twice_in_a_day_is_one_point_with_the_total_length() {
        let mut entries: Vec<_> = (10..15)
            .flat_map(|d| day_of("walk", d, &[("07:00", "07:20"), ("18:00", "18:40")]))
            .collect();
        entries.extend(day_of("walk", 15, &[("07:00", "07:20")]));
        let all = series(&period("2026-05-01"), &entries, &HashSet::new());
        let length = of(&all, Measure::Length);
        assert_eq!(length.len(), 6);
        assert_eq!(length[0], (10, 60.0));
        assert_eq!(length[5], (15, 20.0));
        // First start and last end of the Day.
        assert_eq!(of(&all, Measure::Start)[0], (10, 420.0));
        assert_eq!(of(&all, Measure::End)[0], (10, 1120.0));
    }

    #[test]
    fn a_night_that_straddles_midnight_is_one_day() {
        // Sleep from 23:30 on the 10th and again at 00:30 on the 11th is one night.
        let mut entries: Vec<_> = (10..16)
            .flat_map(|d| day_of("sleep", d, &[("23:30", "23:59")]))
            .collect();
        entries.extend(day_of("sleep", 11, &[("00:30", "06:30")]));
        let all = series(&period("2026-05-01"), &entries, &HashSet::new());
        let days: Vec<u32> = of(&all, Measure::Length).iter().map(|(d, _)| *d).collect();
        assert_eq!(days, [10, 11, 12, 13, 14, 15]);
        assert_eq!(of(&all, Measure::Length)[0].1, 29.0 + 360.0);
    }

    #[test]
    fn habits_are_chosen_over_the_whole_reach_not_the_last_days_alone() {
        let entries: Vec<_> = (10..16).map(|d| entry("workout", d)).collect();
        let all = series(&period("2026-05-01"), &entries, &HashSet::new());
        assert_eq!(of(&all, Measure::Length).len(), 6);
    }
}
