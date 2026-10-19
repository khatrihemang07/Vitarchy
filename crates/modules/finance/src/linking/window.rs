//! How far around what changed detection looks, and how far it reads to judge it fairly.

use chrono::{Days, NaiveDate};

/// Days around the changed dates in which new Transfers are reported.
pub const FOCUS_DAYS: u64 = 3;
/// Extra days read beyond the focus, so a Transaction at its edge still meets every rival
/// (the rungs look a day either side, and a rival's own partners another).
pub const MARGIN_DAYS: u64 = 4;

/// The dates one detection run reports on, and the wider dates it reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    /// Transfers whose debit falls here are reported.
    pub focus: (NaiveDate, NaiveDate),
    /// Transactions dated here are read.
    pub load: (NaiveDate, NaiveDate),
}

/// The window around changes spanning `from` to `to`.
#[must_use]
pub fn around(from: NaiveDate, to: NaiveDate) -> Window {
    let widen = |days: u64| {
        (
            from.checked_sub_days(Days::new(days))
                .unwrap_or(NaiveDate::MIN),
            to.checked_add_days(Days::new(days))
                .unwrap_or(NaiveDate::MAX),
        )
    };
    Window {
        focus: widen(FOCUS_DAYS),
        load: widen(FOCUS_DAYS + MARGIN_DAYS),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 7, d).unwrap()
    }

    #[test]
    fn the_focus_is_three_days_either_side_and_the_load_wider_still() {
        let w = around(day(10), day(12));
        assert_eq!(w.focus, (day(7), day(15)));
        assert_eq!(w.load, (day(3), day(19)));
    }

    #[test]
    fn dates_at_the_edge_of_time_do_not_overflow() {
        let w = around(NaiveDate::MIN, NaiveDate::MAX);
        assert_eq!(w.focus, (NaiveDate::MIN, NaiveDate::MAX));
    }
}
