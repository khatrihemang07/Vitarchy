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

// ---- time zones ----

fn totals(report: &super::model::Report) -> Vec<i64> {
    report.daily.iter().map(|p| p.seconds).collect()
}

#[test]
fn entries_land_on_the_local_day_in_asia_kolkata() {
    // 20:00Z-22:00Z on the 12th is 01:30-03:30 on the 13th in Kolkata (UTC+5:30).
    // 19:00Z-20:00Z on the 11th is 00:30-01:30 on the 12th: outside a UTC week, inside this one.
    let entries = [
        entry(1, "2026-10-12T20:00:00Z", 2, "late"),
        entry(2, "2026-10-11T19:00:00Z", 1, "early"),
    ];
    let q = query(&[("tz", "Asia/Kolkata")]);
    let report = build_report(&q, &entries, &[]);
    assert_eq!(totals(&report), [3600, 7200, 0, 0, 0, 0, 0]);
    assert_eq!(report.overview.total_s, 3 * 3600);
    assert_eq!(report.overview.days_tracked, 2);
}

#[test]
fn entries_land_on_the_local_day_in_america_los_angeles() {
    // 03:00Z on the 13th is 20:00 on the 12th in Los Angeles (UTC-7 in October).
    // 02:00Z on the 19th is still the 18th there, the last Day of the week.
    let entries = [
        entry(1, "2026-10-13T03:00:00Z", 1, "evening"),
        entry(2, "2026-10-19T02:00:00Z", 1, "next week in UTC"),
    ];
    let q = query(&[("tz", "America/Los_Angeles")]);
    let report = build_report(&q, &entries, &[]);
    assert_eq!(totals(&report), [3600, 0, 0, 0, 0, 0, 3600]);
}

#[test]
fn the_same_entry_is_outside_the_period_in_one_zone_and_inside_in_another() {
    let entries = [entry(1, "2026-10-11T19:00:00Z", 1, "x")];
    assert_eq!(build_report(&query(&[]), &entries, &[]).overview.total_s, 0);
    let kolkata = query(&[("tz", "Asia/Kolkata")]);
    assert_eq!(build_report(&kolkata, &entries, &[]).overview.total_s, 3600);
}

// ---- the boundary between the Period and the Previous Period ----

#[test]
fn an_entry_across_the_boundary_is_clipped_once_per_period_and_never_counted_twice() {
    // 22:00Z on the 11th to 03:00Z on the 12th: 2h in the previous week, 3h in this one.
    let spanning = entry(1, "2026-10-11T22:00:00Z", 5, "spans");
    let both = [spanning];
    let report = build_report(&query(&[]), &both, &both);
    let group = &report.groups[0];
    assert_eq!(group.stats.total_s, 3 * 3600);
    assert_eq!(group.entries[0].duration_s, 3 * 3600);
    assert_eq!(group.stats.per_entry.max_s, 3 * 3600);
    assert_eq!(group.stats.per_entry.min_s, 3 * 3600);
    // The previous side holds the other 2h, so the change is 3h - 2h.
    let change = group.change.unwrap();
    assert_eq!(change.total_s, 3600);
    assert_eq!(change.per_entry.max_s, 3600);
    assert_eq!(report.overview.total_s, 3 * 3600);
    assert_eq!(report.overview.change.total_s, 3600);
}

#[test]
fn an_entry_ending_exactly_at_the_boundary_belongs_to_the_previous_period_only() {
    let before = entry(1, "2026-10-11T22:00:00Z", 2, "x");
    let both = [before];
    let report = build_report(&query(&[]), &both, &both);
    assert_eq!(report.overview.total_s, 0);
    assert!(report.groups.is_empty());
    assert_eq!(report.overview.change.total_s, -7200);
}

#[test]
fn the_previous_period_clips_at_its_own_end() {
    // Starts in the previous week, ends well into this one; only the part before the
    // boundary counts there.
    let long = entry(1, "2026-10-10T00:00:00Z", 72, "long");
    let both = [long];
    let report = build_report(&query(&[]), &both, &both);
    let group = &report.groups[0];
    // The entry runs from the 10th to the 13th, so this week holds the 12th only: 24h.
    assert_eq!(group.stats.total_s, 24 * 3600);
    // Previous: 10th and 11th, 48h.
    assert_eq!(group.change.unwrap().total_s, 24 * 3600 - 48 * 3600);
}

#[test]
fn two_empty_periods_give_an_empty_report() {
    let report = build_report(&query(&[]), &[], &[]);
    assert!(report.groups.is_empty());
    assert_eq!(report.overview.total_s, 0);
    assert_eq!(report.overview.entries, 0);
    assert_eq!(
        report.overview.change,
        super::model::OverviewChange::default()
    );
    assert_eq!(totals(&report), [0; 7]);
}

// ---- matching groups across Periods ----

#[test]
fn a_description_matches_the_previous_period_across_a_case_change() {
    let now = [entry(1, "2026-10-13T08:00:00Z", 2, "OFFICE")];
    let before = [entry(2, "2026-10-06T08:00:00Z", 1, "office")];
    let report = build_report(&query(&[]), &now, &before);
    let group = &report.groups[0];
    assert_eq!(group.label, "OFFICE");
    assert!(!group.is_new);
    assert_eq!(group.change.unwrap().total_s, 3600);
}

#[test]
fn a_renamed_project_still_matches_by_its_id() {
    let mut now = entry(1, "2026-10-13T08:00:00Z", 3, "x");
    now.project_id = Some(5);
    now.project_name = Some("New name".to_owned());
    let mut before = entry(2, "2026-10-06T08:00:00Z", 1, "x");
    before.project_id = Some(5);
    before.project_name = Some("Old name".to_owned());
    let report = build_report(&query(&[("group1", "project")]), &[now], &[before]);
    let group = &report.groups[0];
    assert_eq!(group.label, "New name");
    assert!(!group.is_new);
    assert_eq!(group.change.unwrap().total_s, 2 * 3600);
}
