//! The shape of the Active time across the Period: when it began and ended, and when in the
//! day it happened.

use chrono::{DateTime, Duration, Timelike, Utc};
use chrono_tz::Tz;

use super::clip::Piece;

/// First and last Active moment, and Active seconds by local hour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayShape {
    /// Start of the first Active piece.
    pub first_active: Option<DateTime<Utc>>,
    /// End of the last Active piece.
    pub last_active: Option<DateTime<Utc>>,
    /// Active seconds by local hour of day, 24 values from midnight; a piece crossing an hour
    /// is split between the hours.
    pub hours: [i64; 24],
}

/// Reads the shape from `pieces` (in time order) in `tz`.
#[must_use]
pub fn measure(pieces: &[Piece], tz: Tz) -> DayShape {
    let active: Vec<&Piece> = pieces.iter().filter(|p| p.is_active()).collect();
    let mut hours = [0; 24];
    for piece in &active {
        let mut at = piece.start_at;
        while at < piece.end_at {
            let local = at.with_timezone(&tz);
            let into_hour = i64::from(local.minute() * 60 + local.second());
            let hour_end = at + Duration::seconds(3600 - into_hour)
                - Duration::nanoseconds(i64::from(local.nanosecond()));
            let end = hour_end.min(piece.end_at);
            hours[local.hour() as usize] += (end - at).num_seconds();
            at = end;
        }
    }
    DayShape {
        first_active: active.first().map(|p| p.start_at),
        last_active: active.last().map(|p| p.end_at),
        hours,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::model::Away;
    use crate::timeline_summary::test_support::{active_s, at, piece_s};

    #[test]
    fn first_and_last_active_skip_away_and_unknown_at_the_edges() {
        let pieces = [
            piece_s("Lock", 0, 100, Away::Yes),
            active_s("Code", 100, 200),
            active_s("Mail", 300, 400),
            piece_s("X", 400, 500, Away::Unknown),
        ];
        let shape = measure(&pieces, Tz::UTC);
        assert_eq!(
            (shape.first_active, shape.last_active),
            (Some(at(0, 100)), Some(at(0, 400)))
        );
    }

    #[test]
    fn no_active_time_has_no_first_or_last_and_empty_hours() {
        let shape = measure(&[piece_s("Lock", 0, 100, Away::Yes)], Tz::UTC);
        assert_eq!(
            (shape.first_active, shape.last_active, shape.hours),
            (None, None, [0; 24])
        );
    }

    #[test]
    fn a_piece_crossing_an_hour_is_split_between_the_hours() {
        // 08:50 to 09:20 UTC.
        let shape = measure(
            &[active_s("Code", 8 * 3600 + 3000, 9 * 3600 + 1200)],
            Tz::UTC,
        );
        assert_eq!((shape.hours[8], shape.hours[9]), (600, 1200));
        assert_eq!(shape.hours.iter().sum::<i64>(), 1800);
    }

    #[test]
    fn hours_are_local_to_the_time_zone() {
        // 08:00Z to 08:30Z on 2026-10-17 is 10:00 to 10:30 in Berlin (UTC+2).
        let shape = measure(
            &[active_s("Code", 8 * 3600, 8 * 3600 + 1800)],
            Tz::Europe__Berlin,
        );
        assert_eq!((shape.hours[10], shape.hours[8]), (1800, 0));
    }
}
