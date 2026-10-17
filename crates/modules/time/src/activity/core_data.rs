//! Core Data timestamps, shared by every Source whose app is built on Core Data.

use chrono::{DateTime, Utc};

/// Seconds from the Unix epoch to Core Data's reference date (2001-01-01T00:00:00Z).
const CORE_DATA_EPOCH_UNIX_SECS: i64 = 978_307_200;
const MICROS: f64 = 1_000_000.0;

/// Seconds since the Core Data reference date as a UTC time, or `None` if not finite or out of range.
#[must_use]
pub fn core_data_time(secs: f64) -> Option<DateTime<Utc>> {
    if !secs.is_finite() {
        return None;
    }
    let micros = (secs * MICROS).round();
    // Bounded well inside i64 (about +-292k years) before the cast.
    if micros.abs() > 1.0e17 {
        return None;
    }
    #[allow(clippy::cast_possible_truncation)]
    let micros = micros as i64;
    DateTime::from_timestamp_micros(micros.checked_add(CORE_DATA_EPOCH_UNIX_SECS * 1_000_000)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_core_data_reference_date_is_2001_01_01() {
        assert_eq!(
            core_data_time(0.0).unwrap().to_rfc3339(),
            "2001-01-01T00:00:00+00:00"
        );
    }

    #[test]
    fn fractional_seconds_convert_exactly_to_microseconds() {
        let time = core_data_time(800_000_000.123_456).unwrap();
        assert_eq!(time.timestamp(), 1_778_307_200);
        assert_eq!(time.timestamp_subsec_micros(), 123_456);
    }

    #[test]
    fn non_finite_or_absurd_times_are_rejected() {
        assert!(core_data_time(f64::NAN).is_none());
        assert!(core_data_time(f64::INFINITY).is_none());
        assert!(core_data_time(1.0e300).is_none());
    }
}
