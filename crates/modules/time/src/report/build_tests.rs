//! `build_report` end to end on in-memory entries.

use super::{
    build_report,
    entry::{WindowEntry, fixtures::entry},
    query::ReportQuery,
};

fn query(extra: &[(&str, &str)]) -> ReportQuery {
    let mut pairs = vec![
        ("kind", "custom"),
        ("from", "2026-10-12"),
        ("to", "2026-10-18"),
        ("tz", "UTC"),
    ];
    pairs.extend_from_slice(extra);
    let pairs: Vec<(String, String)> = pairs
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect();
    ReportQuery::parse(&pairs).unwrap()
}

fn tagged(mut e: WindowEntry, tags: &[&str]) -> WindowEntry {
    e.tags = tags.iter().map(|t| (*t).to_owned()).collect();
    e
}

#[test]
fn three_and_four_hours_on_one_day_are_one_day_two_entries() {
    let entries = [
        entry(1, "2026-10-13T08:00:00Z", 3, "Office"),
        entry(2, "2026-10-13T13:00:00Z", 4, "office "),
    ];
    let report = build_report(&query(&[]), &entries, &[]);
    let group = &report.groups[0];
    assert_eq!(report.groups.len(), 1);
    assert_eq!(group.label, "Office");
    assert_eq!((group.stats.days, group.stats.entries), (1, 2));
    assert_eq!(group.stats.per_day.avg_s, 7 * 3600);
    assert_eq!(
        (group.stats.per_entry.min_s, group.stats.per_entry.max_s),
        (3 * 3600, 4 * 3600)
    );
    assert_eq!(group.entries.len(), 2);
}

#[test]
fn a_two_tag_entry_counts_under_each_tag_but_once_in_the_overview() {
    let entries = [tagged(
        entry(1, "2026-10-13T08:00:00Z", 2, "x"),
        &["a", "b"],
    )];
    let report = build_report(&query(&[("group1", "tag")]), &entries, &[]);
    assert_eq!(report.groups.len(), 2);
    assert!(report.groups.iter().all(|g| g.stats.total_s == 7200));
    assert_eq!(
        (report.overview.total_s, report.overview.entries),
        (7200, 1)
    );
}

#[test]
fn a_midnight_entry_shows_on_both_days_and_the_daily_series_has_zeros() {
    let entries = [entry(1, "2026-10-13T23:00:00Z", 2, "x")];
    let report = build_report(&query(&[]), &entries, &[]);
    let seconds: Vec<i64> = report.daily.iter().map(|p| p.seconds).collect();
    assert_eq!(seconds, [0, 3600, 3600, 0, 0, 0, 0]);
    assert_eq!(report.overview.days_tracked, 2);
    assert_eq!(report.overview.avg_per_day_s, 3600);
    assert_eq!(report.groups[0].entries[0].duration_s, 7200);
}

#[test]
fn entries_are_clipped_to_the_period() {
    let entries = [entry(1, "2026-10-11T23:00:00Z", 2, "x")];
    let report = build_report(&query(&[]), &entries, &[]);
    assert_eq!(report.overview.total_s, 3600);
}

#[test]
fn groups_are_compared_with_the_previous_period_and_unseen_ones_are_new() {
    let now = [
        entry(1, "2026-10-13T08:00:00Z", 2, "same"),
        entry(2, "2026-10-14T08:00:00Z", 1, "fresh"),
    ];
    let before = [entry(3, "2026-10-06T08:00:00Z", 1, "same")];
    let report = build_report(&query(&[]), &now, &before);
    let same = report.groups.iter().find(|g| g.label == "same").unwrap();
    let fresh = report.groups.iter().find(|g| g.label == "fresh").unwrap();
    assert_eq!(same.change.unwrap().total_s, 3600);
    assert!(fresh.is_new && fresh.change.is_none());
    assert_eq!(report.overview.change.total_s, 7200);
    assert_eq!(report.period.previous_from.to_string(), "2026-10-05");
}

#[test]
fn groups_are_sorted_by_total_and_nest_for_a_second_grouping() {
    let mut a = entry(1, "2026-10-13T08:00:00Z", 1, "short");
    a.project_id = Some(1);
    a.project_name = Some("P".to_owned());
    let mut b = entry(2, "2026-10-13T10:00:00Z", 3, "long");
    b.project_id = Some(1);
    b.project_name = Some("P".to_owned());
    let report = build_report(
        &query(&[("group1", "project"), ("group2", "description")]),
        &[a, b],
        &[],
    );
    assert_eq!(report.groups.len(), 1);
    let children = &report.groups[0].children;
    assert_eq!(children[0].label, "long");
    assert!(report.groups[0].entries.is_empty() && children[0].entries.len() == 1);
}

#[test]
fn the_filter_applies_before_grouping() {
    let entries = [
        entry(1, "2026-10-13T08:00:00Z", 1, "Office work"),
        entry(2, "2026-10-13T10:00:00Z", 1, "Gym"),
    ];
    let report = build_report(&query(&[("q", "OFFICE")]), &entries, &[]);
    assert_eq!(report.overview.entries, 1);
}
