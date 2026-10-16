//! "Longest office day: Tue 2 Oct, 10 h 12 min": a Day far outside the usual.

use super::inputs::{Inputs, Point, Series};
use super::insight::{Insight, InsightKind, Numbers};
use super::score::gap_effect;
use super::stats::{gap, summarize, whole};

/// A Day this far from the usual is as strong an outlier as it gets.
const FULL_EFFECT_MINUTES: f64 = 120.0;

/// Fewest Days before an outlier means anything.
pub const MIN_OUTLIER_SAMPLES: usize = 6;
/// How many ranges beyond the quartiles a Day must lie (Tukey's fences).
pub const FENCE_FACTOR: f64 = 1.5;
/// The range used for the fences is never smaller than this, so a perfectly steady series
/// does not flag every wobble.
pub const MIN_RANGE_MINUTES: f64 = 5.0;
/// An outlier is also at least this many minutes from the median.
pub const MIN_OUTLIER_MINUTES: f64 = 30.0;

/// The most extreme outlying Day of each series that has one.
#[must_use]
pub fn find(inputs: &Inputs) -> Vec<Insight> {
    inputs
        .series
        .iter()
        .filter_map(|series| most_extreme_day(inputs, series))
        .collect()
}

fn most_extreme_day(inputs: &Inputs, series: &Series) -> Option<Insight> {
    let points = series.points_in(inputs.report);
    if points.len() < MIN_OUTLIER_SAMPLES {
        return None;
    }
    let clock = series.is_clock();
    let values: Vec<f64> = points.iter().map(|p| p.value).collect();
    let median = summarize(clock, &values)?.median;
    // Fences are taken on each Day's distance from the median, so a time of day near midnight
    // is measured the short way round and a length is simply its difference.
    let distances: Vec<f64> = values.iter().map(|v| gap(clock, *v, median)).collect();
    let spread = summarize(false, &distances)?;
    let range = spread.iqr(false).max(MIN_RANGE_MINUTES);
    let (low, high) = (
        spread.p25 - FENCE_FACTOR * range,
        spread.p75 + FENCE_FACTOR * range,
    );
    let (point, distance): (&Point, f64) = points
        .iter()
        .zip(distances.iter().copied())
        .filter(|(_, d)| (*d > high || *d < low) && d.abs() >= MIN_OUTLIER_MINUTES)
        // Earlier Day wins a tie, so the answer does not depend on input order.
        .max_by(|a, b| {
            a.1.abs()
                .total_cmp(&b.1.abs())
                .then_with(|| b.0.date.cmp(&a.0.date))
        })?;
    let numbers = Numbers {
        value: whole(point.value),
        reference: Some(whole(median)),
        delta: Some(whole(distance)),
        samples: i64::try_from(points.len()).unwrap_or(i64::MAX),
        ..Numbers::default()
    };
    let effect = gap_effect(distance, FULL_EFFECT_MINUTES);
    Some(Insight::found(
        InsightKind::Outlier,
        series,
        numbers,
        effect,
        vec![point.date],
    ))
}

#[cfg(test)]
mod tests {
    use super::super::inputs::fixtures::{d, inputs, series};
    use super::super::insight::{InsightKind, Measure};
    use super::*;

    fn found(measure: Measure, values: &[f64]) -> Vec<Insight> {
        let points: Vec<(u32, f64)> = values.iter().zip(1..).map(|(v, d)| (d, *v)).collect();
        find(&inputs(vec![series(measure, &points)]))
    }

    fn with_last(last: f64) -> Vec<f64> {
        vec![100.0, 100.0, 100.0, 120.0, 120.0, 120.0, last]
    }

    #[test]
    fn a_long_day_names_itself_and_its_gap_from_the_usual() {
        let found = found(Measure::Length, &with_last(151.0));
        assert_eq!(found.len(), 1);
        let insight = &found[0];
        assert_eq!(insight.kind, InsightKind::Outlier);
        assert_eq!(insight.days, vec![d(7)]);
        let n = &insight.numbers;
        assert_eq!(
            (n.value, n.reference, n.delta, n.samples),
            (151, Some(120), Some(31), 7)
        );
    }

    #[test]
    fn a_day_exactly_on_the_fence_is_not_an_outlier() {
        // Quartiles 100/120 -> range 20, upper fence at 120 + 30.
        assert!(found(Measure::Length, &with_last(150.0)).is_empty());
    }

    #[test]
    fn a_short_day_is_an_outlier_too() {
        let found = found(
            Measure::Length,
            &[120.0, 120.0, 120.0, 140.0, 140.0, 140.0, 49.0],
        );
        assert_eq!(found[0].numbers.delta, Some(-71));
    }

    #[test]
    fn a_day_must_also_be_30_minutes_from_the_median() {
        // Steady at 60: the fence (range floor 5) is 67.5, but 89 is only 29 away.
        let steady = |last| {
            let mut v = vec![60.0; 6];
            v.push(last);
            v
        };
        assert_eq!(found(Measure::Length, &steady(90.0)).len(), 1);
        assert!(found(Measure::Length, &steady(89.0)).is_empty());
    }

    #[test]
    fn six_days_are_needed() {
        assert!(found(Measure::Length, &[60.0, 60.0, 60.0, 60.0, 200.0]).is_empty());
    }

    #[test]
    fn only_the_most_extreme_day_of_a_series_is_reported() {
        let found = found(
            Measure::Length,
            &[60.0, 60.0, 60.0, 60.0, 60.0, 60.0, 120.0, 200.0],
        );
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].days, vec![d(8)]);
    }

    #[test]
    fn a_late_start_is_measured_against_the_usual_start() {
        let mut values = vec![540.0; 6];
        values.push(600.0);
        let found = found(Measure::Start, &values);
        assert_eq!(found[0].numbers.delta, Some(60));
    }

    #[test]
    fn a_start_just_after_midnight_is_close_to_one_before_it() {
        let mut values = vec![1380.0; 6]; // 23:00
        values.push(30.0); // 00:30, 90 minutes later
        let found = found(Measure::Start, &values);
        assert_eq!(found[0].numbers.delta, Some(90));
    }
}
