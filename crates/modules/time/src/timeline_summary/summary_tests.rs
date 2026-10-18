//! Tests for `summarise`.
#![allow(clippy::float_cmp)] // the expected shares are exact binary fractions

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

use super::{model::TimelineSummary, summarise};
use crate::{
    activity::Source,
    report::{
        entry::fixtures,
        period::{Period, PeriodKind},
    },
    timeline::model::{Away, Segment},
};

fn t(text: &str) -> DateTime<Utc> {
    text.parse().unwrap()
}

fn d(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

/// Summarises at `NOW`, with the Usual's history starting at the first given Segment.
fn summarise_from_first(
    period: &Period,
    segments: &[Segment],
    entries: &[crate::report::entry::WindowEntry],
) -> TimelineSummary {
    summarise(
        period,
        segments,
        entries,
        t(NOW),
        segments.first().map(|s| s.start_at),
    )
}

fn segment(start: &str, end: &str, app: &str, away: Away) -> Segment {
    Segment {
        start_at: t(start),
        end_at: t(end),
        app: app.into(),
        title: "w".into(),
        url: None,
        away,
        sources: vec![Source::Toggl],
    }
}

fn day(date: &str) -> Period {
    Period::new(PeriodKind::Day, d(date), d(date), Tz::UTC).unwrap()
}

fn week() -> Period {
    Period::new(PeriodKind::Week, d("2026-10-12"), d("2026-10-18"), Tz::UTC).unwrap()
}

const NOW: &str = "2026-10-20T12:00:00Z";

#[test]
fn a_day_adds_up_active_away_unknown_switches_and_focus() {
    let segments = [
        segment(
            "2026-10-17T09:00:00Z",
            "2026-10-17T09:30:00Z",
            "Code",
            Away::No,
        ),
        segment(
            "2026-10-17T09:30:00Z",
            "2026-10-17T09:40:00Z",
            "Mail",
            Away::No,
        ),
        segment(
            "2026-10-17T09:40:00Z",
            "2026-10-17T10:00:00Z",
            "Lock",
            Away::Yes,
        ),
        segment(
            "2026-10-17T10:00:00Z",
            "2026-10-17T10:05:00Z",
            "Code",
            Away::Unknown,
        ),
    ];
    let entries = [fixtures::entry(1, "2026-10-17T09:00:00Z", 1, "Work")];

    let summary = summarise_from_first(&day("2026-10-17"), &segments, &entries);

    let o = &summary.overview;
    assert_eq!(
        (o.active_secs, o.away_secs, o.unknown_secs),
        (2400, 1200, 300)
    );
    assert_eq!((o.switches, o.longest_focus_secs), (1, 1800));
    assert_eq!(o.switches_per_active_hour, Some(1.5));
    assert_eq!(o.focus_share, 0.75);
    assert_eq!(o.first_active, Some(t("2026-10-17T09:00:00Z")));
    assert_eq!(o.last_active, Some(t("2026-10-17T09:40:00Z")));
    assert_eq!(summary.apps[0].app, "Code");
    assert_eq!(summary.hours[9], 2400);
    assert_eq!(summary.days.len(), 1);
    assert_eq!(summary.segments.as_ref().unwrap().len(), 4);
    assert_eq!(summary.entries.as_ref().unwrap().len(), 1);
}

#[test]
fn a_week_has_no_segments_or_entries_but_one_total_per_day() {
    let segments = [segment(
        "2026-10-13T09:00:00Z",
        "2026-10-13T10:00:00Z",
        "Code",
        Away::No,
    )];
    let summary = summarise_from_first(&week(), &segments, &[]);
    assert!(summary.segments.is_none() && summary.entries.is_none());
    let active: Vec<i64> = summary.days.iter().map(|x| x.active_secs).collect();
    assert_eq!(active, [0, 3600, 0, 0, 0, 0, 0]);
}

#[test]
fn an_empty_period_is_all_zeros_with_empty_lists() {
    let summary = summarise_from_first(&day("2026-10-17"), &[], &[]);
    let o = &summary.overview;
    assert_eq!(
        (
            o.active_secs,
            o.switches,
            o.longest_focus_secs,
            o.focus_share
        ),
        (0, 0, 0, 0.0)
    );
    assert_eq!((o.switches_per_active_hour, o.first_active), (None, None));
    assert!(summary.usual.is_none() && summary.apps.is_empty());
    assert_eq!(summary.hours, vec![0; 24]);
    assert_eq!(summary.segments, Some(vec![]));
}

#[test]
fn the_usual_appears_only_with_history_before_the_period() {
    let history = [
        segment(
            "2026-10-05T09:00:00Z",
            "2026-10-05T10:00:00Z",
            "Code",
            Away::No,
        ),
        segment(
            "2026-10-06T09:00:00Z",
            "2026-10-06T11:00:00Z",
            "Code",
            Away::No,
        ),
    ];
    let current = segment(
        "2026-10-13T09:00:00Z",
        "2026-10-13T10:00:00Z",
        "Code",
        Away::No,
    );
    let all = [history[0].clone(), history[1].clone(), current.clone()];

    let with = summarise_from_first(&week(), &all, &[]);
    let usual = with.usual.expect("history exists");
    assert_eq!((usual.days, usual.active.per_day.avg_s), (2, 5400));
    assert_eq!(with.period.usual_available_days, 7);

    let without = summarise_from_first(&week(), &[current], &[]);
    assert!(without.usual.is_none());
}

#[test]
fn a_segment_crossing_midnight_counts_on_both_days() {
    let segments = [segment(
        "2026-10-17T23:00:00Z",
        "2026-10-18T01:00:00Z",
        "Code",
        Away::No,
    )];
    let period = Period::new(
        PeriodKind::Custom,
        d("2026-10-17"),
        d("2026-10-18"),
        Tz::UTC,
    )
    .unwrap();
    let summary = summarise_from_first(&period, &segments, &[]);
    let active: Vec<i64> = summary.days.iter().map(|x| x.active_secs).collect();
    assert_eq!(active, [3600, 3600]);
    assert_eq!(summary.overview.longest_focus_secs, 7200);
}
