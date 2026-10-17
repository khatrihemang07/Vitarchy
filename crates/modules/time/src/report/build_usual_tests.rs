//! `build_report` comparing with the Usual.

use chrono::{DateTime, Utc};

use super::{
    Context, Entries,
    entry::{WindowEntry, fixtures::entry},
    model::Report,
    query::ReportQuery,
};

fn at(text: &str) -> DateTime<Utc> {
    text.parse().unwrap()
}

/// A Report built a week after the Period ends, by someone tracking since January.
fn build_report(
    q: &ReportQuery,
    current: &[WindowEntry],
    previous: &[WindowEntry],
    usual: &[WindowEntry],
) -> Report {
    let context = Context {
        now: at("2026-10-25T10:00:00Z"),
        first_entry: Some(at("2026-01-01T09:00:00Z")),
    };
    build_report_in(q, current, previous, usual, &context)
}

fn build_report_in(
    q: &ReportQuery,
    current: &[WindowEntry],
    previous: &[WindowEntry],
    usual: &[WindowEntry],
    context: &Context,
) -> Report {
    let entries = Entries {
        current,
        previous,
        usual,
    };
    super::build_report(q, &entries, context)
}

fn query(kind: &str, from: &str, to: &str, extra: &[(&str, &str)]) -> ReportQuery {
    let mut pairs = vec![("kind", kind), ("from", from), ("to", to), ("tz", "UTC")];
    pairs.extend_from_slice(extra);
    let pairs: Vec<(String, String)> = pairs
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect();
    ReportQuery::parse(&pairs).unwrap()
}

/// Two hours of "work" on each of four Days inside 2026-09-14..2026-10-11.
fn usual_work() -> Vec<WindowEntry> {
    ["2026-09-15", "2026-09-22", "2026-09-29", "2026-10-06"]
        .iter()
        .zip(100..)
        .map(|(day, id)| entry(id, &format!("{day}T08:00:00Z"), 2, "work"))
        .collect()
}

#[test]
fn the_period_info_carries_the_usual_range() {
    let q = query("week", "2026-10-12", "2026-10-18", &[]);
    let period = build_report(&q, &[], &[], &[]).period;
    assert_eq!(period.usual_from.to_string(), "2026-09-14");
    assert_eq!(period.usual_to.to_string(), "2026-10-11");
}

#[test]
fn a_week_is_compared_with_a_quarter_of_the_usual() {
    let q = query("week", "2026-10-12", "2026-10-18", &[]);
    let now = [entry(1, "2026-10-13T08:00:00Z", 5, "work")];
    let report = build_report(&q, &now, &[], &usual_work());
    let o = report.overview;
    assert_eq!(o.usual_days, 4);
    // Usual: 8h over 4 Days; a week is 7/28 of that: 2h, 1 entry, 1 Day.
    assert_eq!(o.vs_usual.total_s, 5 * 3600 - 2 * 3600);
    assert_eq!((o.vs_usual.entries, o.vs_usual.days_tracked), (0, 0));
    assert_eq!(o.vs_usual.avg_per_day_s, 5 * 3600 - 2 * 3600);
    let vs = report.groups[0].vs_usual;
    assert_eq!(vs.change.unwrap().total_s, 3 * 3600);
    assert_eq!(vs.usual.unwrap().days, 4);
}

#[test]
fn a_day_is_compared_with_the_usual_average_per_day() {
    let q = query("day", "2026-10-15", "2026-10-15", &[]);
    let now = [entry(1, "2026-10-15T08:00:00Z", 1, "work")];
    let report = build_report(&q, &now, &[], &usual_work());
    let vs = report.groups[0].vs_usual;
    assert_eq!(vs.change.unwrap().per_day.avg_s, 3600 - 2 * 3600);
    assert_eq!(vs.usual.unwrap().per_day.avg_s, 2 * 3600);
}

#[test]
fn a_description_missing_from_the_usual_is_new_even_if_the_previous_period_had_it() {
    let q = query("week", "2026-10-12", "2026-10-18", &[]);
    let now = [entry(1, "2026-10-13T08:00:00Z", 1, "fresh")];
    let before = [entry(2, "2026-10-06T08:00:00Z", 1, "fresh")];
    let group = &build_report(&q, &now, &before, &usual_work()).groups[0];
    assert!(group.vs_usual.is_new && group.vs_usual.change.is_none());
    assert!(!group.vs_previous.is_new);
}

#[test]
fn the_filter_applies_to_the_usual_too() {
    let q = query("week", "2026-10-12", "2026-10-18", &[("q", "gym")]);
    let now = [entry(1, "2026-10-13T08:00:00Z", 1, "gym")];
    let report = build_report(&q, &now, &[], &usual_work());
    assert_eq!(report.overview.usual_days, 0);
    assert!(report.groups[0].vs_usual.is_new);
}

#[test]
fn usual_entries_outside_the_usual_range_are_ignored() {
    let q = query("week", "2026-10-12", "2026-10-18", &[]);
    let now = [entry(1, "2026-10-13T08:00:00Z", 1, "work")];
    let stray = [entry(9, "2026-09-10T08:00:00Z", 1, "work")];
    assert_eq!(build_report(&q, &now, &[], &stray).overview.usual_days, 0);
}

#[test]
fn the_period_info_carries_the_elapsed_and_available_days() {
    let q = query("week", "2026-10-12", "2026-10-18", &[]);
    let context = Context {
        now: at("2026-10-14T10:00:00Z"),
        first_entry: Some(at("2026-10-02T08:00:00Z")),
    };
    let period = build_report_in(&q, &[], &[], &[], &context).period;
    assert_eq!((period.elapsed_days, period.usual_available_days), (3, 10));
}

#[test]
fn a_week_in_progress_is_compared_with_the_usual_for_the_days_so_far() {
    // A two-week Custom Period of which 7 Days have passed: 7/28 of the Usual's 8h is 2h.
    // Scaled to all 14 Days it would be 4h and the 3h tracked would look down.
    let q = query("custom", "2026-10-12", "2026-10-25", &[]);
    let now = [entry(1, "2026-10-13T08:00:00Z", 3, "work")];
    let context = Context {
        now: at("2026-10-18T10:00:00Z"),
        first_entry: Some(at("2026-01-01T09:00:00Z")),
    };
    let report = build_report_in(&q, &now, &[], &usual_work(), &context);
    assert_eq!(report.overview.vs_usual.total_s, 3 * 3600 - 2 * 3600);
    assert_eq!(
        report.groups[0].vs_usual.change.unwrap().total_s,
        3 * 3600 - 2 * 3600
    );
}

#[test]
fn a_period_that_has_not_started_shows_no_change_from_the_usual() {
    let q = query("week", "2026-10-12", "2026-10-18", &[]);
    let now = [entry(1, "2026-10-13T08:00:00Z", 3, "work")];
    let context = Context {
        now: at("2026-10-10T10:00:00Z"),
        first_entry: Some(at("2026-01-01T09:00:00Z")),
    };
    let report = build_report_in(&q, &now, &[], &usual_work(), &context);
    assert_eq!(report.period.elapsed_days, 0);
    assert_eq!(report.overview.vs_usual.total_s, 0);
    assert_eq!(report.groups[0].vs_usual.change.unwrap().total_s, 0);
}

#[test]
fn a_short_history_is_scaled_by_the_days_available_not_28() {
    // Tracking began on 2026-10-02, so only 10 Days of the Usual's range exist; 10h of
    // work in them is 1h a Day, and a week of 7 Days is compared with 7h, not with 2.5h.
    let q = query("week", "2026-10-12", "2026-10-18", &[]);
    let now = [entry(1, "2026-10-13T08:00:00Z", 8, "work")];
    let usual = [entry(2, "2026-10-06T08:00:00Z", 10, "work")];
    let context = Context {
        now: at("2026-10-25T10:00:00Z"),
        first_entry: Some(at("2026-10-02T08:00:00Z")),
    };
    let report = build_report_in(&q, &now, &[], &usual, &context);
    assert_eq!(report.period.usual_available_days, 10);
    assert_eq!(report.overview.vs_usual.total_s, 8 * 3600 - 7 * 3600);
}

#[test]
fn without_history_every_group_is_new_and_the_overview_shows_no_change() {
    let q = query("week", "2026-10-12", "2026-10-18", &[]);
    let now = [entry(1, "2026-10-13T08:00:00Z", 8, "work")];
    let context = Context {
        now: at("2026-10-25T10:00:00Z"),
        first_entry: Some(at("2026-10-12T08:00:00Z")),
    };
    let report = build_report_in(&q, &now, &[], &[], &context);
    assert_eq!(report.period.usual_available_days, 0);
    assert!(report.groups[0].vs_usual.is_new);
    assert_eq!(report.overview.vs_usual.total_s, 0);
}
