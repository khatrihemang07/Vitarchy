//! Tests for `query`.

use super::{filter::Filter, grouping::Grouping, period::PeriodError, query::*};

fn pairs(raw: &[(&str, &str)]) -> Vec<(String, String)> {
    raw.iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}
const BASE: [(&str, &str); 4] = [
    ("kind", "custom"),
    ("from", "2026-10-01"),
    ("to", "2026-10-05"),
    ("tz", "Europe/Berlin"),
];
fn with(extra: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs(&[&BASE[..], extra].concat())
}

#[test]
fn defaults_to_grouping_by_description_with_an_open_filter() {
    let q = ReportQuery::parse(&pairs(&BASE)).unwrap();
    assert_eq!((q.group1, q.group2), (Grouping::Description, None));
    assert_eq!(q.filter, Filter::default());
}

#[test]
fn repeated_names_collect_into_sets() {
    let q = ReportQuery::parse(&with(&[
        ("project", "1"),
        ("project", "2"),
        ("tag", "focus"),
        ("client", "9"),
        ("q", "off"),
        ("group1", "project"),
        ("group2", "tag"),
    ]))
    .unwrap();
    assert_eq!(q.filter.projects.len(), 2);
    assert!(q.filter.tags.contains("focus") && q.filter.clients.contains(&9));
    assert_eq!(q.filter.search.as_deref(), Some("off"));
    assert_eq!(q.groupings(), [Grouping::Project, Grouping::Tag]);
}

#[test]
fn bad_requests_are_named_without_their_values() {
    let err = |extra: &[(&str, &str)]| ReportQuery::parse(&with(extra)).unwrap_err();
    assert_eq!(err(&[("tz", "Mars/Base")]), QueryError::Invalid("tz"));
    assert_eq!(err(&[("from", "10/01")]), QueryError::Invalid("from"));
    assert_eq!(err(&[("project", "x")]), QueryError::Invalid("project"));
    assert_eq!(err(&[("group1", "colour")]), QueryError::Invalid("group1"));
    assert_eq!(err(&[("group2", "description")]), QueryError::SameGrouping);
    assert_eq!(
        err(&[("from", "2026-11-01")]),
        QueryError::Period(PeriodError::Reversed)
    );
    assert_eq!(
        ReportQuery::parse(&pairs(&[("kind", "day")])).unwrap_err(),
        QueryError::Missing("tz")
    );
}

#[test]
fn the_kind_must_fit_the_dates() {
    let err = ReportQuery::parse(&with(&[("kind", "day")])).unwrap_err();
    assert_eq!(err, QueryError::Period(PeriodError::KindMismatch));
}
