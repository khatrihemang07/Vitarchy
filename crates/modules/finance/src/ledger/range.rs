//! Date ranges: how long one is and the range just before it.

use chrono::{Days, NaiveDate};

/// The longest range the Overview answers for (a leap year).
pub const MAX_DAYS: i64 = 366;

/// How many days `from..=to` covers.
#[must_use]
pub fn days(from: NaiveDate, to: NaiveDate) -> i64 {
    (to - from).num_days() + 1
}

/// The range of the same length ending the day before `from`, or `None` past the calendar's
/// start. It is what the Overview compares a Period with.
#[must_use]
pub fn previous_range(from: NaiveDate, to: NaiveDate) -> Option<(NaiveDate, NaiveDate)> {
    let length = u64::try_from(days(from, to)).ok()?;
    let previous_to = from.checked_sub_days(Days::new(1))?;
    let previous_from = previous_to.checked_sub_days(Days::new(length - 1))?;
    Some((previous_from, previous_to))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, month, day).unwrap()
    }

    #[test]
    fn a_single_day_covers_one_day() {
        assert_eq!(days(d(7, 5), d(7, 5)), 1);
    }

    #[test]
    fn the_previous_range_has_the_same_length_and_ends_the_day_before() {
        assert_eq!(previous_range(d(7, 8), d(7, 14)), Some((d(7, 1), d(7, 7))));
        // A 31-day month is compared with the 31 days before it, not with the calendar month.
        assert_eq!(
            previous_range(d(10, 1), d(10, 31)),
            Some((d(8, 31), d(9, 30)))
        );
    }
}
