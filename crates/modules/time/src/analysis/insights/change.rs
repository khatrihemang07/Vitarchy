//! "Commute took 12 min longer than the four weeks before": the last four weeks against the four before.

use super::inputs::{Inputs, Series};
use super::insight::{Insight, InsightKind, Numbers};
use super::score::gap_effect;
use super::stats::{summarize, whole};

/// A change of this many minutes is as strong as a change gets.
const FULL_EFFECT_MINUTES: f64 = 60.0;

/// Fewest Days each window needs.
pub const MIN_WINDOW_SAMPLES: usize = 3;
/// A change counts when the medians differ by at least this many minutes...
pub const MIN_CHANGE_MINUTES: f64 = 10.0;
/// ...and by at least this percent of the earlier median.
pub const MIN_CHANGE_PERCENT: f64 = 15.0;

/// Each length that changed between the earlier window and the recent one.
#[must_use]
pub fn find(inputs: &Inputs) -> Vec<Insight> {
    inputs
        .series
        .iter()
        .filter(|series| !series.is_clock())
        .filter_map(|series| changed(inputs, series))
        .collect()
}

fn changed(inputs: &Inputs, series: &Series) -> Option<Insight> {
    let recent = series.values_in(inputs.recent);
    let earlier = series.values_in(inputs.earlier);
    if recent.len() < MIN_WINDOW_SAMPLES || earlier.len() < MIN_WINDOW_SAMPLES {
        return None;
    }
    let now = summarize(false, &recent)?.median;
    let before = summarize(false, &earlier)?.median;
    let delta = now - before;
    if delta.abs() < MIN_CHANGE_MINUTES || delta.abs() * 100.0 < before * MIN_CHANGE_PERCENT {
        return None;
    }
    let numbers = Numbers {
        value: whole(now),
        reference: Some(whole(before)),
        delta: Some(whole(delta)),
        samples: i64::try_from(recent.len().min(earlier.len())).unwrap_or(i64::MAX),
        ..Numbers::default()
    };
    let effect = gap_effect(delta, FULL_EFFECT_MINUTES);
    Some(Insight::found(
        InsightKind::Change,
        series,
        numbers,
        effect,
        Vec::new(),
    ))
}

#[cfg(test)]
mod tests {
    use super::super::inputs::fixtures::{inputs, series};
    use super::super::insight::{InsightKind, Measure};
    use super::*;

    /// `earlier` values fall on May 1-3 and `recent` values on May 4 onwards.
    fn found(measure: Measure, earlier: &[f64], recent: &[f64]) -> Vec<Insight> {
        let mut points: Vec<(u32, f64)> = earlier.iter().zip(1..).map(|(v, d)| (d, *v)).collect();
        points.extend(recent.iter().zip(4..).map(|(v, d)| (d, *v)));
        find(&inputs(vec![series(measure, &points)]))
    }

    #[test]
    fn a_longer_recent_length_is_reported_against_the_earlier_one() {
        let found = found(Measure::Length, &[60.0; 3], &[75.0; 3]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, InsightKind::Change);
        let n = &found[0].numbers;
        assert_eq!(
            (n.value, n.reference, n.delta, n.samples),
            (75, Some(60), Some(15), 3)
        );
    }

    #[test]
    fn a_shorter_recent_length_has_a_negative_delta() {
        let found = found(Measure::Length, &[60.0; 3], &[40.0; 3]);
        assert_eq!(found[0].numbers.delta, Some(-20));
    }

    #[test]
    fn a_change_must_be_10_minutes() {
        // 60 -> 70 is 16.7%, 60 -> 69 is 9 minutes.
        assert_eq!(found(Measure::Length, &[60.0; 3], &[70.0; 3]).len(), 1);
        assert!(found(Measure::Length, &[60.0; 3], &[69.0; 3]).is_empty());
    }

    #[test]
    fn a_change_must_be_15_percent() {
        // 100 -> 115 is 15%, 100 -> 114 is 14%.
        assert_eq!(found(Measure::Length, &[100.0; 3], &[115.0; 3]).len(), 1);
        assert!(found(Measure::Length, &[100.0; 3], &[114.0; 3]).is_empty());
    }

    #[test]
    fn each_window_needs_three_days() {
        assert!(found(Measure::Length, &[60.0; 2], &[90.0; 3]).is_empty());
        assert!(found(Measure::Length, &[60.0; 3], &[90.0; 2]).is_empty());
    }

    #[test]
    fn times_of_day_are_not_compared() {
        assert!(found(Measure::Start, &[60.0; 3], &[90.0; 3]).is_empty());
    }
}
