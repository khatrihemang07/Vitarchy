//! "It usually takes 12 min (10 to 15 min)": a length that is steady enough to be a habit.

use super::inputs::{Inputs, MIN_SAMPLES, Series};
use super::insight::{Insight, InsightKind, Numbers};
use super::stats::{summarize, whole};

/// A length is a habit only when the middle half of its Days spans at most this share of the median.
pub const MAX_RELATIVE_IQR: f64 = 0.5;

/// The usual length of each series whose lengths are tightly grouped.
#[must_use]
pub fn find(inputs: &Inputs) -> Vec<Insight> {
    inputs
        .series
        .iter()
        .filter(|series| !series.is_clock())
        .filter_map(|series| habit(inputs, series))
        .collect()
}

fn habit(inputs: &Inputs, series: &Series) -> Option<Insight> {
    let values = series.values_in(inputs.report);
    if values.len() < MIN_SAMPLES {
        return None;
    }
    let summary = summarize(false, &values)?;
    if summary.median <= 0.0 {
        return None;
    }
    let relative = summary.iqr(false) / summary.median;
    if relative > MAX_RELATIVE_IQR {
        return None;
    }
    let numbers = Numbers {
        value: whole(summary.median),
        p25: Some(whole(summary.p25)),
        p75: Some(whole(summary.p75)),
        samples: i64::try_from(values.len()).unwrap_or(i64::MAX),
        ..Numbers::default()
    };
    let effect = 1.0 - relative / MAX_RELATIVE_IQR;
    Some(Insight::found(
        InsightKind::TypicalDuration,
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

    fn found(measure: Measure, values: &[f64]) -> Vec<Insight> {
        let points: Vec<(u32, f64)> = values.iter().zip(1..).map(|(v, d)| (d, *v)).collect();
        find(&inputs(vec![series(measure, &points)]))
    }

    #[test]
    fn a_steady_length_gives_its_median_and_range() {
        let found = found(Measure::Length, &[60.0, 60.0, 60.0, 90.0, 90.0]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].kind, InsightKind::TypicalDuration);
        let n = &found[0].numbers;
        assert_eq!(
            (n.value, n.p25, n.p75, n.samples),
            (60, Some(60), Some(90), 5)
        );
    }

    #[test]
    fn a_range_of_half_the_median_is_steady_and_a_bit_more_is_not() {
        assert_eq!(
            found(Measure::Length, &[60.0, 60.0, 60.0, 90.0, 90.0]).len(),
            1
        );
        assert!(found(Measure::Length, &[60.0, 60.0, 60.0, 91.0, 91.0]).is_empty());
    }

    #[test]
    fn four_days_are_too_few() {
        assert!(found(Measure::Length, &[60.0; 4]).is_empty());
    }

    #[test]
    fn times_of_day_are_not_lengths() {
        assert!(found(Measure::Start, &[60.0; 6]).is_empty());
    }

    #[test]
    fn a_length_of_zero_is_not_a_habit() {
        assert!(found(Measure::Length, &[0.0; 6]).is_empty());
    }
}
