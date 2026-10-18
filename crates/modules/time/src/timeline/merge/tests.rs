use super::*;
use crate::timeline::test_support::{at, obs, real};

fn url_obs(source: Source, start: i64, end: i64, app: &str, url: &str) -> Observation {
    let mut o = obs(source, start, end, app, Some(false));
    o.url = Some(url.to_owned());
    o
}

#[test]
fn no_observations_make_an_empty_timeline() {
    assert!(merge(&[], real).is_empty());
}

#[test]
fn one_source_is_followed_as_it_recorded() {
    let only = [obs(Source::Clockify, 0, 10, "A", Some(true))];
    let segments = merge(&only, real);
    assert_eq!(segments.len(), 1);
    let s = &segments[0];
    assert_eq!((s.app.as_str(), s.away), ("A", Away::Yes));
    assert_eq!(s.sources, [Source::Clockify]);
}

#[test]
fn boundaries_and_title_follow_toggl_while_url_and_away_follow_clockify() {
    let mut toggl = obs(Source::Toggl, 0, 10, "Brave", None);
    toggl.title = "Clean title".into();
    let mut clockify = url_obs(Source::Clockify, 0, 10, "Brave", "https://example.com");
    clockify.title = "Clean title; document".into();
    clockify.away = Some(true);
    let segments = merge(&[toggl, clockify], real);
    assert_eq!(segments.len(), 1);
    let s = &segments[0];
    assert_eq!(s.title, "Clean title");
    assert_eq!(s.url.as_deref(), Some("https://example.com"));
    assert_eq!(s.away, Away::Yes);
    assert_eq!(s.sources, [Source::Toggl, Source::Clockify]);
}

#[test]
fn a_stretch_only_toggl_covers_has_unknown_away_and_no_url() {
    let segments = merge(&[obs(Source::Toggl, 0, 10, "A", None)], real);
    assert_eq!(
        (segments[0].away, segments[0].url.clone()),
        (Away::Unknown, None)
    );
}

#[test]
fn app_disagreement_follows_the_boundaries_source() {
    let both = [
        obs(Source::Toggl, 0, 10, "FromToggl", None),
        obs(Source::Clockify, 0, 10, "FromClockify", Some(false)),
    ];
    assert_eq!(merge(&both, real)[0].app, "FromToggl");
}

#[test]
fn a_gap_nothing_covers_stays_a_gap() {
    let segments = merge(
        &[
            obs(Source::Toggl, 0, 10, "A", None),
            obs(Source::Toggl, 20, 30, "A", None),
        ],
        real,
    );
    assert_eq!(segments.len(), 2);
    assert_eq!((segments[0].end_at, segments[1].start_at), (at(10), at(20)));
}

#[test]
fn partial_overlap_splits_at_every_edge_and_lists_who_covered_each_piece() {
    let segments = merge(
        &[
            obs(Source::Toggl, 0, 10, "A", None),
            obs(Source::Clockify, 5, 15, "A", Some(false)),
        ],
        real,
    );
    let shape: Vec<_> = segments
        .iter()
        .map(|s| (s.start_at, s.end_at, s.away, s.sources.len()))
        .collect();
    assert_eq!(
        shape,
        [
            (at(0), at(5), Away::Unknown, 1),
            (at(5), at(10), Away::No, 2),
            (at(10), at(15), Away::No, 1),
        ]
    );
}

#[test]
fn strengths_are_data_so_a_stronger_clockify_would_win_the_boundaries() {
    let swapped = |source: Source| {
        let mut strengths = real(source);
        strengths.boundaries = Some(if source == Source::Clockify { 9 } else { 1 });
        strengths
    };
    let both = [
        obs(Source::Toggl, 0, 10, "FromToggl", None),
        obs(Source::Clockify, 0, 10, "FromClockify", Some(false)),
    ];
    assert_eq!(merge(&both, swapped)[0].app, "FromClockify");
}

#[test]
fn equal_ranks_go_to_the_source_listed_first() {
    let equal = |_: Source| Strengths {
        boundaries: Some(1),
        url: Some(1),
        away: Some(1),
    };
    let both = [
        obs(Source::Clockify, 0, 10, "FromClockify", Some(false)),
        obs(Source::Toggl, 0, 10, "FromToggl", Some(true)),
    ];
    let first = merge(&both, equal);
    assert_eq!(
        (first[0].app.as_str(), first[0].away),
        ("FromToggl", Away::Yes)
    );
}

#[test]
fn a_source_with_no_ranks_still_shows_what_it_saw_when_nothing_else_does() {
    let unranked = |_: Source| Strengths::default();
    let segments = merge(&[obs(Source::Toggl, 0, 10, "A", Some(true))], unranked);
    assert_eq!(
        (segments[0].app.as_str(), segments[0].away),
        ("A", Away::Unknown)
    );
}

#[test]
fn a_url_is_never_attached_to_another_app_than_the_one_that_won() {
    let both = [
        obs(Source::Toggl, 0, 10, "Slack", None),
        url_obs(Source::Clockify, 0, 10, "Brave", "https://example.com"),
    ];
    let s = &merge(&both, real)[0];
    assert_eq!((s.app.as_str(), s.url.clone()), ("Slack", None));
}
