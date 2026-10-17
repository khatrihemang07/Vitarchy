//! Pure: turns Clockify Snapshot rows into [`Activity`], counting what it must leave out.

use serde_json::{Value, json};

use crate::activity::{
    core_data::core_data_time,
    dedupe::dedupe_latest_end,
    model::{Activity, Extras, Parsed},
    source::Source,
};

/// One `ZCDAUTOTRACKERITEM` row exactly as stored, minus the app icon (`ZICONDATA`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RawRow {
    /// `Z_PK`.
    pub pk: Option<i64>,
    /// `Z_ENT`.
    pub ent: Option<i64>,
    /// `Z_OPT`.
    pub opt: Option<i64>,
    /// `ZITEMNO`: a running number, not unique.
    pub item_no: Option<i64>,
    /// `ZTIMEENTRYADDED` (0 or 1).
    pub time_entry_added: Option<i64>,
    /// `ZIDLETIME`, seconds.
    pub idle_time: Option<f64>,
    /// `ZTIMESTARTED`, seconds since the Core Data reference date.
    pub started: Option<f64>,
    /// `ZTIMEENDED`, same unit.
    pub ended: Option<f64>,
    /// `ZCOLOR`: the app's colour in Clockify.
    pub color: Option<String>,
    /// `ZICONID`.
    pub icon_id: Option<String>,
    /// `ZID`: unique per item (Clockify writes the start time here as ISO 8601).
    pub id: Option<String>,
    /// `ZITEMDESCRIPTION`: the window title.
    pub description: Option<String>,
    /// `ZITEMURL`: the page address, for browsers.
    pub url: Option<String>,
    /// `ZNAME`: the app's name.
    pub name: Option<String>,
}

/// One row to an Activity, or `None` when its id, times or span are unusable.
#[must_use]
pub fn parse_row(row: &RawRow) -> Option<Activity> {
    let source_id = row.id.clone().filter(|id| !id.is_empty())?;
    let start_at = core_data_time(row.started?)?;
    let end_at = core_data_time(row.ended?)?;
    if end_at < start_at {
        return None;
    }
    Some(Activity {
        source: Source::Clockify,
        raw: raw_json(row),
        source_id,
        app: row.name.clone().unwrap_or_default(),
        title: row.description.clone().unwrap_or_default(),
        start_at,
        end_at,
        extras: Extras {
            url: row.url.clone().filter(|url| !url.is_empty()),
            idle_secs: Some(row.idle_time.filter(|secs| secs.is_finite()).unwrap_or(0.0)),
            ..Extras::default()
        },
    })
}

fn raw_json(row: &RawRow) -> Value {
    json!({
        "Z_PK": row.pk, "Z_ENT": row.ent, "Z_OPT": row.opt,
        "ZITEMNO": row.item_no, "ZTIMEENTRYADDED": row.time_entry_added,
        "ZIDLETIME": row.idle_time, "ZTIMESTARTED": row.started, "ZTIMEENDED": row.ended,
        "ZCOLOR": row.color, "ZICONID": row.icon_id, "ZID": row.id,
        "ZITEMDESCRIPTION": row.description, "ZITEMURL": row.url, "ZNAME": row.name,
    })
}

/// Parses every row; see [`dedupe_latest_end`] for repeated ids.
#[must_use]
pub fn parse_all(rows: &[RawRow]) -> Parsed {
    dedupe_latest_end(rows.iter().map(parse_row))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: &str) -> RawRow {
        RawRow {
            pk: Some(1),
            idle_time: Some(0.0),
            started: Some(0.0),
            ended: Some(90.0),
            name: Some("Code".into()),
            description: Some("README.md".into()),
            url: Some(String::new()),
            id: Some(id.into()),
            ..RawRow::default()
        }
    }

    #[test]
    fn a_row_keeps_its_names_and_its_original_numbers_in_raw() {
        let activity = parse_row(&row("a")).unwrap();
        assert_eq!(activity.source_id, "a");
        assert_eq!(
            (activity.app.as_str(), activity.title.as_str()),
            ("Code", "README.md")
        );
        assert_eq!(activity.end_at.to_rfc3339(), "2001-01-01T00:01:30+00:00");
        assert_eq!(activity.raw["ZTIMEENDED"], 90.0);
        assert_eq!(activity.raw["ZID"], "a");
    }

    #[test]
    fn an_empty_url_is_no_url() {
        assert_eq!(parse_row(&row("a")).unwrap().extras.url, None);
        let mut browser = row("b");
        browser.url = Some("https://example.com".into());
        assert_eq!(
            parse_row(&browser).unwrap().extras.url.as_deref(),
            Some("https://example.com")
        );
    }

    #[test]
    fn idle_time_is_kept_as_reported_even_when_longer_than_the_span() {
        let mut idle = row("a");
        idle.idle_time = Some(500.0);
        assert_eq!(parse_row(&idle).unwrap().extras.idle_secs, Some(500.0));
    }

    #[test]
    fn rows_without_an_id_or_with_a_negative_span_are_rejected() {
        let mut empty_id = row("");
        empty_id.id = Some(String::new());
        let mut backwards = row("b");
        backwards.ended = Some(-5.0);
        let mut no_start = row("c");
        no_start.started = None;
        assert!(parse_row(&empty_id).is_none());
        assert!(parse_row(&backwards).is_none());
        assert!(parse_row(&no_start).is_none());
    }

    #[test]
    fn a_repeated_id_keeps_the_later_end_and_counts_the_rest_as_skipped() {
        let mut later = row("a");
        later.ended = Some(200.0);
        let mut bad = row("b");
        bad.id = None;
        let parsed = parse_all(&[row("a"), later, bad]);
        assert_eq!(parsed.activities.len(), 1);
        assert_eq!(parsed.activities[0].raw["ZTIMEENDED"], 200.0);
        assert_eq!(parsed.skipped, 2);
    }
}
