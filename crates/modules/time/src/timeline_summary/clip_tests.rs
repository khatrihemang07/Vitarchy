//! Tests for `clip`.

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

use super::clip::*;
use crate::{
    activity::Source,
    report::period::{Period, PeriodKind},
    timeline::model::{Away, Segment},
};

fn t(text: &str) -> DateTime<Utc> {
    text.parse().unwrap()
}

fn segment(start: &str, end: &str) -> Segment {
    Segment {
        start_at: t(start),
        end_at: t(end),
        app: "Code".into(),
        title: "main.rs".into(),
        url: Some("https://a.dev/x".into()),
        away: Away::No,
        sources: vec![Source::Toggl],
    }
}

fn day(tz: Tz, date: &str) -> Period {
    let d: NaiveDate = date.parse().unwrap();
    Period::new(PeriodKind::Day, d, d, tz).unwrap()
}

#[test]
fn a_segment_inside_the_day_is_one_piece_with_its_details() {
    let pieces = clip(
        &[segment("2026-10-17T09:00:00Z", "2026-10-17T10:00:00Z")],
        &day(Tz::UTC, "2026-10-17"),
    );
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].secs(), 3600);
    assert_eq!(
        (pieces[0].app.as_str(), pieces[0].url.as_deref()),
        ("Code", Some("https://a.dev/x"))
    );
    assert!(pieces[0].is_active());
}

#[test]
fn a_segment_crossing_local_midnight_is_split_between_the_days() {
    let period = Period::new(
        PeriodKind::Custom,
        "2026-10-17".parse().unwrap(),
        "2026-10-18".parse().unwrap(),
        Tz::Europe__Berlin,
    )
    .unwrap();
    // Berlin is UTC+2 in October: local midnight is 22:00Z.
    let pieces = clip(
        &[segment("2026-10-17T21:00:00Z", "2026-10-17T23:00:00Z")],
        &period,
    );
    let shape: Vec<_> = pieces
        .iter()
        .map(|p| (p.date.to_string(), p.secs()))
        .collect();
    assert_eq!(
        shape,
        [
            ("2026-10-17".to_owned(), 3600),
            ("2026-10-18".to_owned(), 3600)
        ]
    );
    assert_eq!(pieces[0].end_at, pieces[1].start_at);
}

#[test]
fn the_period_edges_cut_segments_and_outside_ones_vanish() {
    let period = day(Tz::UTC, "2026-10-17");
    let pieces = clip(
        &[
            segment("2026-10-16T23:00:00Z", "2026-10-17T01:00:00Z"),
            segment("2026-10-17T23:00:00Z", "2026-10-18T02:00:00Z"),
            segment("2026-10-19T00:00:00Z", "2026-10-19T01:00:00Z"),
        ],
        &period,
    );
    let secs: Vec<_> = pieces.iter().map(Piece::secs).collect();
    assert_eq!(secs, [3600, 3600]);
}
