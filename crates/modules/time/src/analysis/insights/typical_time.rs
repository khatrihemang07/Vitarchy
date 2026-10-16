//! "You usually start at 9:36 (9:31 to 9:41)": a start or end that is steady enough to be a habit.

use super::inputs::MIN_SAMPLES;
use super::inputs::{Inputs, Series};
use super::insight::{Insight, InsightKind, Numbers};
use super::stats::{summarize, whole};

/// A time of day is a habit only when the middle half of its Days fit in this many minutes.
pub const MAX_IQR_MINUTES: f64 = 45.0;

/// The usual start or end of each series whose times are tightly grouped.
#[must_use]
pub fn find(inputs: &Inputs) -> Vec<Insight> {
    inputs
        .series
        .iter()
        .filter(|series| series.is_clock())
        .filter_map(|series| habit(inputs, series))
        .collect()
}

fn habit(inputs: &Inputs, series: &Series) -> Option<Insight> {
    let values = series.values_in(inputs.report);
    if values.len() < MIN_SAMPLES {
        return None;
    }
    let summary = summarize(true, &values)?;
    let range = summary.iqr(true);
    if range > MAX_IQR_MINUTES {
        return None;
    }
    let numbers = Numbers {
        value: whole(summary.median),
        p25: Some(whole(summary.p25)),
        p75: Some(whole(summary.p75)),
        samples: i64::try_from(values.len()).unwrap_or(i64::MAX),
        ..Numbers::default()
    };
    // The tighter the habit, the stronger the finding.
    let effect = 1.0 - range / MAX_IQR_MINUTES;
    Some(Insight::found(
        InsightKind::TypicalTime,
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
    fn a_steady_start_gives_its_usual_time_and_range() {
        let found = found(Measure::Start, &[540.0, 540.0, 540.0, 585.0, 585.0]);
        assert_eq!(found.len(), 1);
        let insight = &found[0];
        assert_eq!(insight.kind, InsightKind::TypicalTime);
        assert_eq!(insight.measure, Measure::Start);
        let n = &insight.numbers;
        assert_eq!(
            (n.value, n.p25, n.p75, n.samples),
            (540, Some(540), Some(585), 5)
        );
    }

    #[test]
    fn a_range_of_exactly_45_minutes_is_a_habit_and_one_minute_more_is_not() {
        assert_eq!(
            found(Measure::End, &[540.0, 540.0, 540.0, 585.0, 585.0]).len(),
            1
        );
        assert!(found(Measure::End, &[540.0, 540.0, 540.0, 586.0, 586.0]).is_empty());
    }

    #[test]
    fn four_days_are_too_few() {
        assert!(found(Measure::Start, &[540.0; 4]).is_empty());
    }

    #[test]
    fn days_outside_the_period_do_not_count() {
        let mut s = series(
            Measure::Start,
            &[(1, 540.0), (2, 540.0), (3, 540.0), (4, 540.0), (5, 540.0)],
        );
        for p in s.points.iter_mut().skip(2) {
            p.date = p.date + chrono::Days::new(40); // into June
        }
        assert!(find(&inputs(vec![s])).is_empty());
    }

    #[test]
    fn lengths_are_not_times_of_day() {
        assert!(found(Measure::Length, &[540.0; 6]).is_empty());
    }

    #[test]
    fn times_either_side_of_midnight_form_one_habit() {
        let found = found(Measure::Start, &[1410.0, 1430.0, 10.0, 30.0, 60.0]);
        let n = &found[0].numbers;
        assert_eq!((n.value, n.p25, n.p75), (10, Some(1430), Some(30)));
    }
}
