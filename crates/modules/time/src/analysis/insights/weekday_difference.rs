//! "On Fridays you leave office 50 min earlier than usual": a weekday set apart from the rest.

use std::collections::BTreeMap;

use chrono::Datelike;

use super::inputs::{Inputs, MIN_SAMPLES, Series};
use super::insight::{Insight, InsightKind, Numbers};
use super::score::gap_effect;
use super::stats::{Summary, gap, summarize, whole};

/// A gap of this many minutes is as strong as a gap gets.
const FULL_EFFECT_MINUTES: f64 = 60.0;

/// Fewest Days a weekday needs before it is compared.
pub const MIN_WEEKDAY_SAMPLES: usize = 3;
/// A weekday differs when its median is at least this many minutes from the overall median...
pub const MIN_DIFFERENCE_MINUTES: f64 = 20.0;
/// ...and, for lengths, at least this percent of it (for times of day: at least the overall range).
pub const MIN_DIFFERENCE_PERCENT: f64 = 15.0;

/// The weekday that differs most from the rest, for each series that has one.
#[must_use]
pub fn find(inputs: &Inputs) -> Vec<Insight> {
    inputs
        .series
        .iter()
        .filter_map(|series| most_different_weekday(inputs, series))
        .collect()
}

/// A weekday's figures against the overall ones.
struct Candidate {
    weekday: u8,
    median: f64,
    delta: f64,
    samples: usize,
}

fn most_different_weekday(inputs: &Inputs, series: &Series) -> Option<Insight> {
    let clock = series.is_clock();
    let points = series.points_in(inputs.report);
    if points.len() < MIN_SAMPLES {
        return None;
    }
    let all: Vec<f64> = points.iter().map(|p| p.value).collect();
    let overall = summarize(clock, &all)?;
    let mut by_weekday: BTreeMap<u8, Vec<f64>> = BTreeMap::new();
    for point in &points {
        let weekday = u8::try_from(point.date.weekday().num_days_from_monday()).unwrap_or(0);
        by_weekday.entry(weekday).or_default().push(point.value);
    }
    let best = by_weekday
        .into_iter()
        .filter(|(_, values)| values.len() >= MIN_WEEKDAY_SAMPLES)
        .filter_map(|(weekday, values)| {
            let median = summarize(clock, &values)?.median;
            let delta = gap(clock, median, overall.median);
            differs(clock, delta, &overall).then_some(Candidate {
                weekday,
                median,
                delta,
                samples: values.len(),
            })
        })
        .max_by(|a, b| a.delta.abs().total_cmp(&b.delta.abs()))?;
    let numbers = Numbers {
        value: whole(best.median),
        reference: Some(whole(overall.median)),
        delta: Some(whole(best.delta)),
        weekday: Some(best.weekday),
        samples: i64::try_from(best.samples).unwrap_or(i64::MAX),
        ..Numbers::default()
    };
    let effect = gap_effect(best.delta, FULL_EFFECT_MINUTES);
    Some(Insight::found(
        InsightKind::WeekdayDifference,
        series,
        numbers,
        effect,
        Vec::new(),
    ))
}

/// Whether a weekday's gap from the overall median is big enough. A length must also be
/// [`MIN_DIFFERENCE_PERCENT`] of the median; a time of day, which a percentage means nothing
/// for, must exceed the range its Days already vary over.
fn differs(clock: bool, delta: f64, overall: &Summary) -> bool {
    let size = delta.abs();
    let big_enough = if clock {
        size >= overall.iqr(true)
    } else {
        size * 100.0 >= overall.median * MIN_DIFFERENCE_PERCENT
    };
    size >= MIN_DIFFERENCE_MINUTES && big_enough
}

#[cfg(test)]
mod tests {
    use super::super::inputs::fixtures::{inputs, series};
    use super::super::insight::{InsightKind, Measure};
    use super::*;

    /// 2026-05-04 is a Monday. Days 5-7 (Tue-Thu) and 11-14, 18-21 take `usual`; each
    /// `(day, value)` in `odd` replaces that day.
    fn found(measure: Measure, usual: f64, odd: &[(u32, f64)]) -> Vec<Insight> {
        let days = [5, 6, 7, 11, 12, 13, 14, 18, 19, 20, 21];
        let mut points: Vec<(u32, f64)> = days.iter().map(|d| (*d, usual)).collect();
        points.extend_from_slice(odd);
        find(&inputs(vec![series(measure, &points)]))
    }

    /// Three Fridays (May 1, 8, 15).
    fn fridays(value: f64) -> Vec<(u32, f64)> {
        [1, 8, 15].iter().map(|d| (*d, value)).collect()
    }

    #[test]
    fn a_weekday_far_from_the_rest_is_reported_with_its_gap() {
        let found = found(Measure::Length, 600.0, &fridays(500.0));
        assert_eq!(found.len(), 1);
        let insight = &found[0];
        assert_eq!(insight.kind, InsightKind::WeekdayDifference);
        let n = &insight.numbers;
        assert_eq!(
            (n.weekday, n.value, n.reference, n.delta, n.samples),
            (Some(4), 500, Some(600), Some(-100), 3)
        );
    }

    #[test]
    fn a_length_must_differ_by_15_percent() {
        // 15% of 600 is 90.
        assert_eq!(found(Measure::Length, 600.0, &fridays(510.0)).len(), 1);
        assert!(found(Measure::Length, 600.0, &fridays(511.0)).is_empty());
    }

    #[test]
    fn a_length_must_differ_by_20_minutes() {
        // 15% of 100 is 15, so 20 minutes is the stricter rule.
        assert_eq!(found(Measure::Length, 100.0, &fridays(80.0)).len(), 1);
        assert!(found(Measure::Length, 100.0, &fridays(81.0)).is_empty());
    }

    #[test]
    fn a_weekday_needs_three_days() {
        let two = fridays(400.0)[..2].to_vec();
        assert!(found(Measure::Length, 600.0, &two).is_empty());
    }

    #[test]
    fn a_time_of_day_is_compared_by_minutes_the_short_way_round() {
        let early = found(Measure::End, 1050.0, &fridays(1000.0));
        let n = &early[0].numbers;
        assert_eq!((n.weekday, n.delta), (Some(4), Some(-50)));
        assert!(found(Measure::End, 1050.0, &fridays(1035.0)).is_empty());
    }

    #[test]
    fn only_the_most_different_weekday_of_a_series_is_reported() {
        let mondays: Vec<(u32, f64)> = [4, 11, 18].iter().map(|d| (*d, 740.0)).collect();
        let mut odd = fridays(480.0);
        odd.extend(mondays);
        let found = found(Measure::Length, 600.0, &odd);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].numbers.weekday, Some(0));
    }
}
