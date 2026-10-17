//! A Routine's later Steps may run past midnight into the early hours (`dinner → bath → sleep`
//! with sleep logged at 00:10). This is the one place that decides what counts as "early".
//!
//! An entry belongs to the Day it **starts** in. An entry that starts on the next Day before
//! [`SPILL_UNTIL`] (local time) may still finish the previous Day's Routine; matching and
//! suggesting both read the Days through [`by_day`] so they apply the same rule.

use std::collections::BTreeMap;

use chrono::{DateTime, Days, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;

use crate::analysis::{Period, lifelog::LifeEntry};

/// Local time of day before which an entry starting on the next Day may still finish a Routine.
/// It is compared as local wall-clock time, so a Day with a clock change needs no special case.
pub const SPILL_UNTIL: NaiveTime = NaiveTime::from_hms_opt(6, 0, 0).unwrap();

/// Whole Days of entries loaded either side of a Period so Days at its edges spill correctly:
/// the Day before (to know which early entries it used) and the Day after (to spill into).
/// Two covers a 25-hour Day.
const MARGIN_DAYS: i64 = 2;

/// An entry and its position in the slice it came from (to tell used entries apart).
#[derive(Clone, Copy)]
pub(super) struct Slot<'a> {
    pub index: usize,
    pub entry: &'a LifeEntry,
}

/// The instants between which entries must be loaded for `period`'s Routines to be exact.
#[must_use]
pub fn load_window(period: &Period) -> (DateTime<Utc>, DateTime<Utc>) {
    let margin = chrono::Duration::days(MARGIN_DAYS);
    (period.start() - margin, period.end() + margin)
}

/// Whether `entry` starts early enough on its Day to finish the Routine of the Day before.
pub(super) fn is_early(tz: Tz, entry: &LifeEntry) -> bool {
    entry.start.with_timezone(&tz).time() < SPILL_UNTIL
}

/// The local date after `date`.
pub(super) fn next_day(date: NaiveDate) -> NaiveDate {
    date.checked_add_days(Days::new(1)).unwrap_or(date)
}

/// The Period's first and last local date.
pub(super) fn dates(period: &Period) -> (NaiveDate, NaiveDate) {
    let first = period.days().first().map_or(NaiveDate::MIN, |d| d.date);
    let last = period.days().last().map_or(NaiveDate::MAX, |d| d.date);
    (first, last)
}

/// The entries of each local date from the Day before `period` to the Day after it, each Day's
/// in start order.
pub(super) fn by_day<'a>(
    period: &Period,
    entries: impl IntoIterator<Item = (usize, &'a LifeEntry)>,
) -> BTreeMap<NaiveDate, Vec<Slot<'a>>> {
    let tz = period.tz();
    let (first, last) = dates(period);
    let (first, last) = (first.pred_opt().unwrap_or(first), next_day(last));
    let mut days: BTreeMap<NaiveDate, Vec<Slot>> = BTreeMap::new();
    for (index, entry) in entries {
        let date = entry.start.with_timezone(&tz).date_naive();
        if (first..=last).contains(&date) {
            days.entry(date).or_default().push(Slot { index, entry });
        }
    }
    for day in days.values_mut() {
        day.sort_by_key(|slot| slot.entry.start);
    }
    days
}
