//! Finding the Day the Previous Period should reach back to.

use chrono::NaiveDate;

use super::{
    entry::WindowEntry,
    filter::Filter,
    items,
    period::{MAX_DAYS, Period, PeriodKind},
};

/// The last local Day inside `range` with time from an entry matching `filter`. Goes through
/// `items::prepare`, so an entry crossing midnight counts for the Day it reaches.
#[must_use]
pub fn latest_tracked_day(
    entries: &[WindowEntry],
    filter: &Filter,
    range: &Period,
) -> Option<NaiveDate> {
    items::prepare(entries, filter, range)
        .iter()
        .flat_map(|item| item.parts.iter().map(|part| part.date))
        .max()
}

/// The [`MAX_DAYS`] Days ending the Day before `immediate` starts: how far back the search
/// for a tracked Day goes, the same cap a Period's own length has.
#[must_use]
pub fn lookback_range(immediate: &Period) -> Period {
    Period {
        kind: PeriodKind::Custom,
        from: immediate.from - chrono::Duration::days(MAX_DAYS),
        to: immediate.from - chrono::Duration::days(1),
        tz: immediate.tz,
    }
}
