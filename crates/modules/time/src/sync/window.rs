//! Pure: which Time Entries a Sync asks Toggl for.

use chrono::{DateTime, Duration, Utc};

use crate::toggl::types::EntryQuery;

/// How far back the first Sync reaches; Toggl's `/me/time_entries` allows about 3 months.
pub const FIRST_SYNC_DAYS: i64 = 90;
/// A cursor older than this is not trusted: Toggl rejects a `since` older than
/// about 3 months, which would fail every Sync forever. Ten days of slack under
/// [`FIRST_SYNC_DAYS`] covers the overlap, clock skew and a Sync that runs late.
pub const STALE_CURSOR_DAYS: i64 = FIRST_SYNC_DAYS - 10;
/// Later Syncs start a little before the cursor so clock skew between this
/// computer and Toggl cannot hide a change. Re-reading is free: writes are upserts.
pub const OVERLAP_SECS: i64 = 60;

/// The request for a Sync starting at `now`, given the previous Sync's cursor.
/// Without a usable cursor (none, or older than [`STALE_CURSOR_DAYS`]) it reads
/// the whole recent window, which also lets the Sync notice deletions.
#[must_use]
pub fn window(cursor: Option<DateTime<Utc>>, now: DateTime<Utc>) -> EntryQuery {
    match cursor.filter(|cursor| *cursor > now - Duration::days(STALE_CURSOR_DAYS)) {
        None => EntryQuery::Range {
            start_date: (now - Duration::days(FIRST_SYNC_DAYS)).date_naive(),
            end_date: (now + Duration::days(1)).date_naive(),
        },
        Some(cursor) => EntryQuery::Since((cursor - Duration::seconds(OVERLAP_SECS)).timestamp()),
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone};

    use super::*;

    fn at(y: i32, m: u32, d: u32, h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap()
    }

    #[test]
    fn without_a_cursor_it_asks_for_the_last_90_days_up_to_tomorrow() {
        let query = window(None, at(2026, 4, 11, 12));
        assert_eq!(
            query,
            EntryQuery::Range {
                start_date: NaiveDate::from_ymd_opt(2026, 1, 11).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2026, 4, 12).unwrap(),
            }
        );
    }

    #[test]
    fn with_a_cursor_it_asks_for_changes_since_just_before_it() {
        let cursor = at(2026, 4, 1, 8);
        let query = window(Some(cursor), at(2026, 4, 11, 12));
        assert_eq!(query, EntryQuery::Since(cursor.timestamp() - OVERLAP_SECS));
    }

    #[test]
    fn a_cursor_older_than_the_limit_falls_back_to_the_full_window() {
        let now = at(2026, 4, 11, 12);
        let stale = now - Duration::days(STALE_CURSOR_DAYS) - Duration::seconds(1);
        assert!(matches!(window(Some(stale), now), EntryQuery::Range { .. }));
    }

    #[test]
    fn a_cursor_just_inside_the_limit_still_asks_for_changes() {
        let now = at(2026, 4, 11, 12);
        let fresh = now - Duration::days(STALE_CURSOR_DAYS) + Duration::seconds(1);
        assert_eq!(
            window(Some(fresh), now),
            EntryQuery::Since(fresh.timestamp() - OVERLAP_SECS)
        );
    }
}
