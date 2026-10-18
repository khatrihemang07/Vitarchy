//! Tests for `apps`.
#![allow(clippy::float_cmp)] // the expected shares are exact binary fractions

use super::apps::*;
use super::clip::Piece;
use super::test_support::{active_s, piece_s};
use crate::timeline::model::Away;

fn with(mut piece: Piece, title: &str, url: Option<&str>) -> Piece {
    piece.title = title.to_owned();
    piece.url = url.map(str::to_owned);
    piece
}

#[test]
fn apps_rank_by_active_time_with_shares_and_ignore_away_and_unknown() {
    let pieces = [
        active_s("Mail", 0, 100),
        active_s("Code", 100, 400),
        piece_s("Lock", 400, 900, Away::Yes),
        piece_s("Other", 900, 950, Away::Unknown),
    ];
    let apps = rank(&pieces);
    let shape: Vec<_> = apps
        .iter()
        .map(|a| (a.app.as_str(), a.active_secs))
        .collect();
    assert_eq!(shape, [("Code", 300), ("Mail", 100)]);
    assert_eq!(apps[0].share, 0.75);
    assert_eq!(apps[1].share, 0.25);
}

#[test]
fn ties_break_by_name() {
    let apps = rank(&[active_s("b", 0, 10), active_s("a", 10, 20)]);
    let names: Vec<_> = apps.iter().map(|a| a.app.as_str()).collect();
    assert_eq!(names, ["a", "b"]);
}

#[test]
fn titles_are_summed_ranked_and_capped_at_ten() {
    let mut pieces = vec![
        with(active_s("Code", 0, 5), "same", None),
        with(active_s("Code", 5, 10), "same", None),
    ];
    for n in 0..12 {
        let start = 10 + n * 20;
        pieces.push(with(
            active_s("Code", start, start + 1 + n),
            &format!("t{n:02}"),
            None,
        ));
    }
    let titles = &rank(&pieces)[0].titles;
    assert_eq!(titles.len(), TOP_PER_APP);
    assert_eq!(
        (titles[0].title.as_str(), titles[0].active_secs),
        ("t11", 12)
    );
    assert!(titles.iter().all(|t| t.title != "t00"));
    assert!(
        titles
            .iter()
            .any(|t| t.title == "same" && t.active_secs == 10)
    );
}

#[test]
fn sites_group_urls_by_domain_and_skip_unparseable_ones() {
    let pieces = [
        with(
            active_s("Safari", 0, 30),
            "a",
            Some("https://www.docs.rs/x"),
        ),
        with(active_s("Safari", 30, 40), "b", Some("https://docs.rs/y")),
        with(active_s("Safari", 40, 100), "c", Some("https://zed.dev")),
        with(active_s("Safari", 100, 150), "d", Some("garbage")),
        with(active_s("Safari", 150, 160), "e", None),
    ];
    let sites: Vec<_> = rank(&pieces)[0]
        .sites
        .iter()
        .map(|s| (s.domain.clone(), s.active_secs))
        .collect();
    assert_eq!(
        sites,
        [("zed.dev".to_owned(), 60), ("docs.rs".to_owned(), 40)]
    );
}
