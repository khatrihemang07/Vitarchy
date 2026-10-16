//! Pure: which Time Entries a Sync asks Toggl for.

use chrono::{DateTime, Duration, Utc};

use crate::toggl::types::EntryQuery;

/// How far back the first Sync reaches; Toggl's `/me/time_entries` allows about 3 months.
pub const FIRST_SYNC_DAYS: i64 = 90;
/// Later Syncs start a little before the cursor so clock skew between this
/// computer and Toggl cannot hide a change. Re-reading is free: writes are upserts.
pub const OVERLAP_SECS: i64 = 60;

/// The request for a Sync starting at `now`, given the previous Sync's cursor.
#[must_use]
pub fn window(cursor: Option<DateTime<Utc>>, now: DateTime<Utc>) -> EntryQuery {
    match cursor {
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
}
