//! Joins touching Segments that say the same thing.

use super::model::Segment;

/// Merges each Segment into its predecessor when they touch and have equal app, title,
/// url, Away and Sources, so a joined Segment never claims a Source that did not cover all of
/// it. Expects `segments` ordered by start.
#[must_use]
pub fn coalesce(segments: Vec<Segment>) -> Vec<Segment> {
    let mut joined: Vec<Segment> = Vec::with_capacity(segments.len());
    for segment in segments {
        match joined.last_mut() {
            Some(last) if joins(last, &segment) => {
                last.end_at = segment.end_at;
            }
            _ => joined.push(segment),
        }
    }
    joined
}

fn joins(earlier: &Segment, later: &Segment) -> bool {
    earlier.end_at == later.start_at
        && earlier.app == later.app
        && earlier.title == later.title
        && earlier.url == later.url
        && earlier.away == later.away
        && earlier.sources == later.sources
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::Source;
    use crate::timeline::{model::Away, test_support::at};

    fn seg(start: i64, end: i64, app: &str, sources: &[Source]) -> Segment {
        Segment {
            start_at: at(start),
            end_at: at(end),
            app: app.to_owned(),
            title: String::new(),
            url: None,
            away: Away::No,
            sources: sources.to_vec(),
        }
    }

    #[test]
    fn equal_touching_neighbours_become_one() {
        let joined = coalesce(vec![
            seg(0, 5, "A", &[Source::Toggl]),
            seg(5, 9, "A", &[Source::Toggl]),
        ]);
        assert_eq!(joined.len(), 1);
        assert_eq!((joined[0].start_at, joined[0].end_at), (at(0), at(9)));
        assert_eq!(joined[0].sources, [Source::Toggl]);
    }

    #[test]
    fn neighbours_backed_by_different_sources_stay_apart_so_none_overstates_its_sources() {
        let out = coalesce(vec![
            seg(0, 5, "A", &[Source::Toggl]),
            seg(5, 9, "A", &[Source::Toggl, Source::Clockify]),
        ]);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].sources, [Source::Toggl]);
    }

    #[test]
    fn a_gap_or_a_difference_keeps_them_apart() {
        let gap = coalesce(vec![
            seg(0, 5, "A", &[Source::Toggl]),
            seg(6, 9, "A", &[Source::Toggl]),
        ]);
        let different = coalesce(vec![
            seg(0, 5, "A", &[Source::Toggl]),
            seg(5, 9, "B", &[Source::Toggl]),
        ]);
        assert_eq!((gap.len(), different.len()), (2, 2));
    }

    #[test]
    fn a_different_away_or_url_keeps_them_apart() {
        let mut away = seg(5, 9, "A", &[Source::Toggl]);
        away.away = Away::Yes;
        let mut url = seg(5, 9, "A", &[Source::Toggl]);
        url.url = Some("https://example.com".into());
        for other in [away, url] {
            let out = coalesce(vec![seg(0, 5, "A", &[Source::Toggl]), other]);
            assert_eq!(out.len(), 2);
        }
    }
}
