//! Seconds by Away, in total and per Day.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use super::{clip::Piece, model::DayTotals};
use crate::timeline::model::Away;

/// Seconds by Away. Unknown is its own bucket and is never added to Active.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Seconds {
    /// Known not Away.
    pub active: i64,
    /// Known Away.
    pub away: i64,
    /// Not known.
    pub unknown: i64,
}

impl std::ops::AddAssign for Seconds {
    fn add_assign(&mut self, other: Self) {
        self.active += other.active;
        self.away += other.away;
        self.unknown += other.unknown;
    }
}

/// Seconds by Away over `pieces`.
#[must_use]
pub fn seconds(pieces: &[Piece]) -> Seconds {
    let mut total = Seconds::default();
    for piece in pieces {
        match piece.away {
            Away::No => total.active += piece.secs(),
            Away::Yes => total.away += piece.secs(),
            Away::Unknown => total.unknown += piece.secs(),
        }
    }
    total
}

/// One entry for each of `days`, zero where no piece falls.
#[must_use]
pub fn per_day(days: &[NaiveDate], pieces: &[Piece]) -> Vec<DayTotals> {
    let mut by_day: BTreeMap<NaiveDate, Seconds> = BTreeMap::new();
    for piece in pieces {
        *by_day.entry(piece.date).or_default() += seconds(std::slice::from_ref(piece));
    }
    days.iter()
        .map(|date| {
            let secs = by_day.get(date).copied().unwrap_or_default();
            DayTotals {
                date: *date,
                active_secs: secs.active,
                away_secs: secs.away,
                unknown_secs: secs.unknown,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline_summary::test_support::{active_s, piece_s};

    #[test]
    fn seconds_split_into_active_away_and_unknown() {
        let pieces = [
            active_s("Code", 0, 100),
            piece_s("Lock", 100, 130, Away::Yes),
            piece_s("X", 130, 140, Away::Unknown),
        ];
        assert_eq!(
            seconds(&pieces),
            Seconds {
                active: 100,
                away: 30,
                unknown: 10
            }
        );
    }

    #[test]
    fn every_day_has_an_entry_and_empty_days_are_zero() {
        let d = |n: u32| NaiveDate::from_ymd_opt(2026, 10, n).unwrap();
        let day_two = 86_400;
        let pieces = [
            active_s("Code", 0, 60),
            piece_s("Lock", day_two, day_two + 30, Away::Yes),
        ];
        let days = per_day(&[d(16), d(17), d(18)], &pieces);
        let shape: Vec<_> = days
            .iter()
            .map(|t| (t.date, t.active_secs, t.away_secs, t.unknown_secs))
            .collect();
        assert_eq!(
            shape,
            [(d(16), 0, 0, 0), (d(17), 60, 0, 0), (d(18), 0, 30, 0)]
        );
    }
}
