//! Pure: turns Snapshot rows into [`Activity`], counting what it must leave out.

use std::{collections::HashMap, fmt::Write};

use chrono::{DateTime, Utc};
use serde_json::{Value, json};

/// Seconds from the Unix epoch to Core Data's reference date (2001-01-01T00:00:00Z).
const CORE_DATA_EPOCH_UNIX_SECS: i64 = 978_307_200;
const MICROS: f64 = 1_000_000.0;

/// One `ZMANAGEDACTIVITY` row exactly as stored.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RawRow {
    /// `Z_PK`.
    pub pk: Option<i64>,
    /// `Z_ENT`.
    pub ent: Option<i64>,
    /// `Z_OPT`.
    pub opt: Option<i64>,
    /// `ZISIDLE` (0 or 1).
    pub is_idle: Option<i64>,
    /// `ZSYNCSTATUS`.
    pub sync_status: Option<i64>,
    /// `ZSTART`, seconds since the Core Data reference date.
    pub start: Option<f64>,
    /// `ZEND`, same unit.
    pub end: Option<f64>,
    /// `ZFILENAME`: the app's name.
    pub app: Option<String>,
    /// `ZTITLE`: the window title.
    pub title: Option<String>,
    /// `ZCLIENT`: 16-byte device id.
    pub client: Option<Vec<u8>>,
    /// `ZID`: 16-byte id, unique per activity.
    pub id: Option<Vec<u8>>,
}

/// An Activity ready to store.
#[derive(Debug, Clone, PartialEq)]
pub struct Activity {
    /// Toggl's id for it, as a UUID string (primary key).
    pub toggl_id: String,
    /// The device that recorded it, when readable.
    pub client_id: Option<String>,
    /// App name.
    pub app: String,
    /// Window title (may be empty).
    pub title: String,
    /// When it began.
    pub start_at: DateTime<Utc>,
    /// When it ended so far.
    pub end_at: DateTime<Utc>,
    /// Whether the computer was idle.
    pub idle: bool,
    /// Every source column.
    pub raw: Value,
}

/// The result of parsing a whole Snapshot.
#[derive(Debug, Default, PartialEq)]
pub struct Parsed {
    /// Distinct, valid Activity.
    pub activities: Vec<Activity>,
    /// Rows left out.
    pub skipped: usize,
}

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

/// A 16-byte blob as a hyphenated lowercase UUID string.
#[must_use]
pub fn uuid_string(bytes: &[u8]) -> Option<String> {
    let bytes: &[u8; 16] = bytes.try_into().ok()?;
    let mut out = String::with_capacity(36);
    for (index, byte) in bytes.iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            out.push('-');
        }
        // Writing to a String cannot fail.
        let _ = write!(out, "{byte:02x}");
    }
    Some(out)
}

/// One row to an Activity, or `None` when its id, times or span are unusable.
#[must_use]
pub fn parse_row(row: &RawRow) -> Option<Activity> {
    let toggl_id = uuid_string(row.id.as_deref()?)?;
    let client_id = row.client.as_deref().and_then(uuid_string);
    let start_at = core_data_time(row.start?)?;
    let end_at = core_data_time(row.end?)?;
    if end_at < start_at {
        return None;
    }
    Some(Activity {
        raw: raw_json(row, &toggl_id, client_id.as_deref()),
        toggl_id,
        client_id,
        app: row.app.clone().unwrap_or_default(),
        title: row.title.clone().unwrap_or_default(),
        start_at,
        end_at,
        idle: row.is_idle.unwrap_or(0) != 0,
    })
}

fn raw_json(row: &RawRow, id: &str, client: Option<&str>) -> Value {
    json!({
        "Z_PK": row.pk, "Z_ENT": row.ent, "Z_OPT": row.opt,
        "ZISIDLE": row.is_idle, "ZSYNCSTATUS": row.sync_status,
        "ZSTART": row.start, "ZEND": row.end,
        "ZFILENAME": row.app, "ZTITLE": row.title,
        "ZCLIENT": client, "ZID": id,
    })
}

/// Parses every row. A repeated id keeps the row that ends latest; the others count as skipped.
#[must_use]
pub fn parse_all(rows: &[RawRow]) -> Parsed {
    let mut position: HashMap<String, usize> = HashMap::new();
    let mut activities: Vec<Activity> = Vec::new();
    let mut skipped = 0;
    for row in rows {
        let Some(activity) = parse_row(row) else {
            skipped += 1;
            continue;
        };
        if let Some(&at) = position.get(&activity.toggl_id) {
            skipped += 1;
            if activity.end_at > activities[at].end_at {
                activities[at] = activity;
            }
        } else {
            position.insert(activity.toggl_id.clone(), activities.len());
            activities.push(activity);
        }
    }
    Parsed {
        activities,
        skipped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(last: u8) -> Vec<u8> {
        let mut bytes = vec![
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 1, 2, 3, 4, 5, 6, 7,
        ];
        bytes.push(last);
        bytes
    }

    fn row(last: u8) -> RawRow {
        RawRow {
            pk: Some(1),
            is_idle: Some(0),
            start: Some(0.0),
            end: Some(90.5),
            app: Some("App".into()),
            title: Some(String::new()),
            client: Some(id(0)),
            id: Some(id(last)),
            ..RawRow::default()
        }
    }

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

    #[test]
    fn sixteen_bytes_make_a_hyphenated_uuid() {
        assert_eq!(
            uuid_string(&id(0xff)).unwrap(),
            "12345678-9abc-def0-0102-0304050607ff"
        );
        assert!(uuid_string(&[1, 2, 3]).is_none());
    }

    #[test]
    fn a_row_keeps_an_empty_title_and_its_original_reals_in_raw() {
        let activity = parse_row(&row(1)).unwrap();
        assert_eq!(activity.title, "");
        assert!(!activity.idle);
        assert_eq!(activity.raw["ZEND"], 90.5);
        assert_eq!(activity.raw["ZID"], "12345678-9abc-def0-0102-030405060701");
    }

    #[test]
    fn an_idle_flag_marks_the_activity_idle() {
        let mut idle = row(1);
        idle.is_idle = Some(1);
        assert!(parse_row(&idle).unwrap().idle);
    }

    #[test]
    fn rows_with_a_bad_id_or_a_negative_span_are_rejected() {
        let mut bad_id = row(1);
        bad_id.id = Some(vec![1, 2]);
        let mut backwards = row(2);
        backwards.end = Some(-5.0);
        let mut no_start = row(3);
        no_start.start = None;
        assert!(parse_row(&bad_id).is_none());
        assert!(parse_row(&backwards).is_none());
        assert!(parse_row(&no_start).is_none());
    }

    #[test]
    fn an_unreadable_client_does_not_reject_the_row() {
        let mut odd = row(1);
        odd.client = Some(vec![9]);
        assert_eq!(parse_row(&odd).unwrap().client_id, None);
    }

    #[test]
    fn a_repeated_id_keeps_the_later_end_and_counts_the_rest_as_skipped() {
        let mut later = row(1);
        later.end = Some(200.0);
        let mut bad = row(2);
        bad.id = None;
        let parsed = parse_all(&[row(1), later, bad]);
        assert_eq!(parsed.activities.len(), 1);
        assert_eq!(parsed.activities[0].raw["ZEND"], 200.0);
        assert_eq!(parsed.skipped, 2);
    }
}
