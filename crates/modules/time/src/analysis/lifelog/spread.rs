//! Typical value and spread (median, quartiles) of durations and of clock times.
//!
//! Clock times live on a circle: sleep that starts between 23:30 and 01:00 must
//! not average to midday. [`clock_spread`] cuts the circle on the side opposite
//! the circular mean, unrolls the times into one run of minutes, takes ordinary
//! quartiles there and rolls the results back.

// Inputs are minutes of a day, seconds of an entry and counts of Days: far inside f64's exact range,
// and every result is non-negative and rounded before it becomes an integer.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use serde::Serialize;
use utoipa::ToSchema;

const MINUTES_PER_DAY: f64 = 1440.0;

/// Median and interquartile range of a length of time.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct DurationSpread {
    /// Seconds, the middle value.
    pub median_seconds: i64,
    /// Seconds, the first quartile.
    pub p25_seconds: i64,
    /// Seconds, the third quartile.
    pub p75_seconds: i64,
}

/// Median and interquartile range of a time of day, as local minutes after midnight (0 to 1439).
/// `p25_minute` may be later than `p75_minute` when the range crosses midnight.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct ClockSpread {
    /// The middle time.
    pub median_minute: u32,
    /// The first quartile, going round the clock from the cut.
    pub p25_minute: u32,
    /// The third quartile.
    pub p75_minute: u32,
}

/// The `q`th quantile (0 to 1) of sorted values, interpolating between neighbours.
fn quantile(sorted: &[f64], q: f64) -> f64 {
    let at = q * (sorted.len() - 1) as f64;
    let (low, high) = (at.floor() as usize, at.ceil() as usize);
    sorted[low] + (sorted[high] - sorted[low]) * (at - low as f64)
}

/// Median and quartiles of `seconds`; `None` when there are none.
#[must_use]
pub fn duration_spread(seconds: &[i64]) -> Option<DurationSpread> {
    if seconds.is_empty() {
        return None;
    }
    let mut sorted: Vec<f64> = seconds.iter().map(|s| *s as f64).collect();
    sorted.sort_by(f64::total_cmp);
    let pick = |q| quantile(&sorted, q).round() as i64;
    Some(DurationSpread {
        median_seconds: pick(0.5),
        p25_seconds: pick(0.25),
        p75_seconds: pick(0.75),
    })
}

/// Median and quartiles of times of day given as minutes after midnight; `None` when there are none.
#[must_use]
pub fn clock_spread(minutes: &[f64]) -> Option<ClockSpread> {
    if minutes.is_empty() {
        return None;
    }
    let cut = cut_point(minutes);
    let mut unrolled: Vec<f64> = minutes
        .iter()
        .map(|m| (m - cut).rem_euclid(MINUTES_PER_DAY))
        .collect();
    unrolled.sort_by(f64::total_cmp);
    let pick = |q| {
        let minute = (quantile(&unrolled, q) + cut)
            .rem_euclid(MINUTES_PER_DAY)
            .round() as u32;
        minute % MINUTES_PER_DAY as u32
    };
    Some(ClockSpread {
        median_minute: pick(0.5),
        p25_minute: pick(0.25),
        p75_minute: pick(0.75),
    })
}

/// Where to cut the clock circle: opposite the circular mean, so a cluster is never split.
/// With no direction to speak of (times spread evenly round the clock) the cut is midnight.
fn cut_point(minutes: &[f64]) -> f64 {
    let angle = |m: &f64| m / MINUTES_PER_DAY * std::f64::consts::TAU;
    let (sin, cos) = minutes.iter().fold((0.0, 0.0), |(s, c), m| {
        (s + angle(m).sin(), c + angle(m).cos())
    });
    let resultant = sin.hypot(cos) / minutes.len() as f64;
    if resultant < 1e-6 {
        return 0.0;
    }
    let mean = sin.atan2(cos) / std::f64::consts::TAU * MINUTES_PER_DAY;
    (mean + MINUTES_PER_DAY / 2.0).rem_euclid(MINUTES_PER_DAY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_values_have_no_spread() {
        assert!(duration_spread(&[]).is_none());
        assert!(clock_spread(&[]).is_none());
    }

    #[test]
    fn durations_give_median_and_interpolated_quartiles() {
        let spread = duration_spread(&[600, 100, 300, 500, 200]).unwrap();
        assert_eq!(
            (
                spread.p25_seconds,
                spread.median_seconds,
                spread.p75_seconds
            ),
            (200, 300, 500)
        );
        let even = duration_spread(&[100, 200, 300, 400]).unwrap();
        assert_eq!(even.median_seconds, 250);
        assert_eq!(even.p25_seconds, 175);
    }

    #[test]
    fn a_single_value_is_its_own_spread() {
        let spread = duration_spread(&[90]).unwrap();
        assert_eq!((spread.p25_seconds, spread.p75_seconds), (90, 90));
    }

    fn clock(minutes: &[f64]) -> (u32, u32, u32) {
        let s = clock_spread(minutes).unwrap();
        (s.p25_minute, s.median_minute, s.p75_minute)
    }

    #[test]
    fn daytime_clock_times_use_plain_quartiles() {
        // 09:00, 09:10, 09:20, 09:30, 09:40
        assert_eq!(clock(&[540.0, 550.0, 560.0, 570.0, 580.0]), (550, 560, 570));
    }

    #[test]
    fn times_either_side_of_midnight_stay_together() {
        // 23:30, 23:50, 00:10, 00:30, 01:00 -> median 00:10, not midday.
        let (p25, median, p75) = clock(&[1410.0, 1430.0, 10.0, 30.0, 60.0]);
        assert_eq!(median, 10);
        assert_eq!(p25, 1430);
        assert_eq!(p75, 30);
    }

    #[test]
    fn input_order_does_not_matter_around_midnight() {
        let a = clock(&[60.0, 1410.0, 30.0, 1430.0, 10.0]);
        assert_eq!(a, clock(&[1410.0, 1430.0, 10.0, 30.0, 60.0]));
    }

    #[test]
    fn evenly_spread_times_still_give_a_deterministic_answer() {
        let spread = clock_spread(&[0.0, 360.0, 720.0, 1080.0]).unwrap();
        assert!(spread.median_minute < 1440);
    }

    #[test]
    fn a_time_exactly_at_midnight_is_zero_not_1440() {
        assert_eq!(clock(&[0.0, 0.0, 0.0]), (0, 0, 0));
    }
}
