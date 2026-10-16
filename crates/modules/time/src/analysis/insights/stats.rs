//! Median, quartiles and differences of minutes, for lengths (a line) and times of day (a circle).

use crate::analysis::lifelog::{clock_spread, quantile};

/// Minutes in a day: the circumference of the clock.
pub const MINUTES_PER_DAY: f64 = 1440.0;

/// The usual value and the range the middle half of the values fall in.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    /// The middle value.
    pub median: f64,
    /// First quartile.
    pub p25: f64,
    /// Third quartile.
    pub p75: f64,
}

impl Summary {
    /// Width of the middle half; for a time of day the range may cross midnight.
    #[must_use]
    pub fn iqr(&self, clock: bool) -> f64 {
        if clock {
            (self.p75 - self.p25).rem_euclid(MINUTES_PER_DAY)
        } else {
            self.p75 - self.p25
        }
    }
}

/// Median and quartiles of `values`; `None` when there are none. With `clock` the values are
/// times of day and a cluster either side of midnight stays together.
#[must_use]
pub fn summarize(clock: bool, values: &[f64]) -> Option<Summary> {
    if values.is_empty() {
        return None;
    }
    if clock {
        let spread = clock_spread(values)?;
        return Some(Summary {
            median: f64::from(spread.median_minute),
            p25: f64::from(spread.p25_minute),
            p75: f64::from(spread.p75_minute),
        });
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    Some(Summary {
        median: quantile(&sorted, 0.5),
        p25: quantile(&sorted, 0.25),
        p75: quantile(&sorted, 0.75),
    })
}

/// `a` minus `b`; for times of day the short way round the clock, so 00:10 minus 23:50 is +20.
#[must_use]
pub fn gap(clock: bool, a: f64, b: f64) -> f64 {
    let difference = a - b;
    if clock {
        (difference + MINUTES_PER_DAY / 2.0).rem_euclid(MINUTES_PER_DAY) - MINUTES_PER_DAY / 2.0
    } else {
        difference
    }
}

/// `minutes` as a whole number, for the figures an Insight carries.
// Minutes here are well inside i64's range and finite.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn whole(minutes: f64) -> i64 {
    minutes.round() as i64
}

#[cfg(test)]
// The expected figures are exact by construction (small whole minutes).
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn lengths_have_a_median_and_quartiles() {
        let s = summarize(false, &[50.0, 10.0, 30.0, 20.0, 40.0]).unwrap();
        assert_eq!((s.p25, s.median, s.p75), (20.0, 30.0, 40.0));
        assert_eq!(s.iqr(false), 20.0);
    }

    #[test]
    fn nothing_has_no_summary() {
        assert!(summarize(false, &[]).is_none());
        assert!(summarize(true, &[]).is_none());
    }

    #[test]
    fn times_either_side_of_midnight_are_one_cluster() {
        let s = summarize(true, &[1410.0, 1430.0, 10.0, 30.0, 60.0]).unwrap();
        assert_eq!((s.p25, s.median, s.p75), (1430.0, 10.0, 30.0));
        assert_eq!(s.iqr(true), 40.0);
    }

    #[test]
    fn a_gap_between_times_goes_the_short_way_round() {
        assert_eq!(gap(true, 10.0, 1430.0), 20.0);
        assert_eq!(gap(true, 1430.0, 10.0), -20.0);
        assert_eq!(gap(true, 600.0, 540.0), 60.0);
        assert_eq!(gap(false, 10.0, 1430.0), -1420.0);
    }
}
