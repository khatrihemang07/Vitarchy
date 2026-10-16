//! The usual times and lengths of a Routine across its Occurrences.
//!
//! Every Step is measured on each Day it happened (incomplete Days included). The door-to-door
//! total, the gaps' totals and the weekday breakdown use complete Occurrences only, so a missing
//! Step never shortens a total.

use chrono::{DateTime, Datelike, Utc};
use chrono_tz::Tz;
use serde::Serialize;
use utoipa::ToSchema;

use super::Occurrence;
use crate::analysis::lifelog::{
    ClockSpread, DurationSpread, clock_spread, duration_spread, minute_of_day,
};

/// The usual start, end and length of one Step.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct StepStat {
    /// Days the Step happened.
    pub days: i64,
    /// When it usually starts, local time.
    pub start: Option<ClockSpread>,
    /// When it usually ends, local time.
    pub end: Option<ClockSpread>,
    /// How long it usually lasts.
    pub duration: Option<DurationSpread>,
}

/// How a Routine goes on one weekday, over complete Occurrences.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct WeekdayStat {
    /// 0 for Monday to 6 for Sunday.
    pub weekday: u8,
    /// Complete Occurrences on this weekday.
    pub days: i64,
    /// Median door-to-door seconds.
    pub door_to_door_median_seconds: i64,
    /// Median seconds of each Step, in Step order.
    pub step_median_seconds: Vec<Option<i64>>,
}

/// Statistics of a Routine over a Period.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct RoutineStats {
    /// Occurrences with every Step.
    pub complete_days: i64,
    /// Occurrences with a Step missing.
    pub incomplete_days: i64,
    /// First Step's start to last Step's end, complete Occurrences only.
    pub door_to_door: Option<DurationSpread>,
    /// One per Step in order.
    pub steps: Vec<StepStat>,
    /// Waits between Steps (one fewer than Steps), over Days where both Steps happened.
    pub gaps: Vec<Option<DurationSpread>>,
    /// Weekdays that have complete Occurrences, Monday first.
    pub weekdays: Vec<WeekdayStat>,
}

/// Statistics for `occurrences` of a Routine with `step_count` Steps, clock times in `tz`.
#[must_use]
pub fn routine_stats(tz: Tz, step_count: usize, occurrences: &[Occurrence]) -> RoutineStats {
    let complete: Vec<&Occurrence> = occurrences.iter().filter(|o| o.complete).collect();
    let door_to_door: Vec<i64> = complete
        .iter()
        .filter_map(|o| o.door_to_door_seconds)
        .collect();
    RoutineStats {
        complete_days: count(complete.len()),
        incomplete_days: count(occurrences.len() - complete.len()),
        door_to_door: duration_spread(&door_to_door),
        steps: (0..step_count)
            .map(|i| step_stat(tz, occurrences, i))
            .collect(),
        gaps: (0..step_count.saturating_sub(1))
            .map(|i| {
                let waits: Vec<i64> = occurrences.iter().filter_map(|o| o.gaps[i]).collect();
                duration_spread(&waits)
            })
            .collect(),
        weekdays: weekdays(step_count, &complete),
    }
}

fn count(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

fn step_stat(tz: Tz, occurrences: &[Occurrence], index: usize) -> StepStat {
    let spans: Vec<_> = occurrences
        .iter()
        .filter_map(|o| o.steps[index].as_ref())
        .collect();
    let clock = |pick: fn(&super::StepSpan) -> DateTime<Utc>| {
        let minutes: Vec<f64> = spans.iter().map(|s| minute_of_day(tz, pick(s))).collect();
        clock_spread(&minutes)
    };
    let seconds: Vec<i64> = spans.iter().map(|s| s.seconds).collect();
    StepStat {
        days: count(spans.len()),
        start: clock(|s| s.start),
        end: clock(|s| s.end),
        duration: duration_spread(&seconds),
    }
}

fn weekdays(step_count: usize, complete: &[&Occurrence]) -> Vec<WeekdayStat> {
    (0..7u8)
        .filter_map(|weekday| {
            let of_day: Vec<&&Occurrence> = complete
                .iter()
                .filter(|o| o.date.weekday().num_days_from_monday() == u32::from(weekday))
                .collect();
            let totals: Vec<i64> = of_day
                .iter()
                .filter_map(|o| o.door_to_door_seconds)
                .collect();
            Some(WeekdayStat {
                weekday,
                days: count(of_day.len()),
                door_to_door_median_seconds: duration_spread(&totals)?.median_seconds,
                step_median_seconds: (0..step_count)
                    .map(|i| {
                        let secs: Vec<i64> = of_day
                            .iter()
                            .filter_map(|o| o.steps[i].as_ref().map(|s| s.seconds))
                            .collect();
                        duration_spread(&secs).map(|s| s.median_seconds)
                    })
                    .collect(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::{MatchKind, Step, entry, occurrences};
    use super::*;
    use crate::analysis::Period;

    fn stats_for(entries: &[crate::analysis::lifelog::LifeEntry], tz: &str) -> RoutineStats {
        let period = Period::parse("2026-05-04", "2026-05-17", tz).unwrap();
        let steps: Vec<Step> = ["to", "in"]
            .map(|v| Step {
                label: v.into(),
                kind: MatchKind::Description,
                value: v.into(),
            })
            .to_vec();
        let found = occurrences(&period, &steps, entries);
        routine_stats(period.tz(), steps.len(), &found)
    }

    /// `to` for `to_min` minutes, 15 minutes wait, `in` for 8 h.
    fn day(d: u32, to_min: i64) -> Vec<crate::analysis::lifelog::LifeEntry> {
        let base: DateTime<Utc> = format!("2026-05-{d:02}T08:00:00Z").parse().unwrap();
        let to_end = base + chrono::Duration::minutes(to_min);
        let in_start = to_end + chrono::Duration::minutes(15);
        let in_end = in_start + chrono::Duration::hours(8);
        let fmt = |t: DateTime<Utc>| t.to_rfc3339();
        vec![
            entry("p", "to", &fmt(base), &fmt(to_end)),
            entry("p", "in", &fmt(in_start), &fmt(in_end)),
        ]
    }

    #[test]
    fn complete_days_give_door_to_door_and_step_medians() {
        let entries: Vec<_> = [(4, 30), (5, 40), (6, 50)]
            .iter()
            .flat_map(|(d, m)| day(*d, *m))
            .collect();
        let stats = stats_for(&entries, "UTC");
        assert_eq!((stats.complete_days, stats.incomplete_days), (3, 0));
        assert_eq!(
            stats.steps[0].duration.as_ref().unwrap().median_seconds,
            40 * 60
        );
        assert_eq!(stats.steps[0].start.as_ref().unwrap().median_minute, 8 * 60);
        assert_eq!(stats.gaps[0].as_ref().unwrap().median_seconds, 900);
        // door to door = to + 15 min + 8 h; median day is the 40 min one.
        assert_eq!(
            stats.door_to_door.unwrap().median_seconds,
            (40 + 15) * 60 + 8 * 3600
        );
    }

    #[test]
    fn incomplete_days_count_per_step_but_not_in_the_total() {
        let mut entries: Vec<_> = day(4, 30);
        entries.extend(day(5, 30));
        entries.push(entry(
            "p",
            "in",
            "2026-05-06T09:00:00Z",
            "2026-05-06T10:00:00Z",
        ));
        let stats = stats_for(&entries, "UTC");
        assert_eq!((stats.complete_days, stats.incomplete_days), (2, 1));
        assert_eq!(stats.steps[0].days, 2);
        assert_eq!(stats.steps[1].days, 3);
        assert_eq!(stats.gaps[0].as_ref().unwrap().median_seconds, 900);
        assert_eq!(
            stats.door_to_door.unwrap().p25_seconds,
            30 * 60 + 15 * 60 + 8 * 3600
        );
    }

    #[test]
    fn weekdays_are_summarised_monday_first_over_complete_days() {
        // 2026-05-04 is a Monday; Mondays 4 and 11 take 30 and 50 min, Tuesday 5 takes 60.
        let entries: Vec<_> = [(4, 30), (11, 50), (5, 60)]
            .iter()
            .flat_map(|(d, m)| day(*d, *m))
            .collect();
        let stats = stats_for(&entries, "UTC");
        let weekdays: Vec<(u8, i64)> = stats.weekdays.iter().map(|w| (w.weekday, w.days)).collect();
        assert_eq!(weekdays, [(0, 2), (1, 1)]);
        assert_eq!(stats.weekdays[0].step_median_seconds[0], Some(40 * 60));
        assert_eq!(
            stats.weekdays[0].door_to_door_median_seconds,
            (40 + 15) * 60 + 8 * 3600
        );
    }

    #[test]
    fn clock_times_are_local() {
        let entries: Vec<_> = day(4, 30);
        let stats = stats_for(&entries, "Asia/Kolkata");
        assert_eq!(
            stats.steps[0].start.as_ref().unwrap().median_minute,
            8 * 60 + 330
        );
    }

    #[test]
    fn no_occurrences_have_no_spreads() {
        let stats = stats_for(&[], "UTC");
        assert_eq!(stats.complete_days, 0);
        assert!(stats.door_to_door.is_none() && stats.weekdays.is_empty());
        assert!(
            stats
                .steps
                .iter()
                .all(|s| s.days == 0 && s.duration.is_none())
        );
        assert_eq!(stats.steps.len(), 2);
        assert_eq!(stats.gaps, [None]);
    }
}
