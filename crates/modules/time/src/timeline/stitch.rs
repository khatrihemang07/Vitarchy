//! Fitting a rebuilt stretch into the stored Timeline without leaving a hole or a seam.

use super::{
    coalesce::coalesce,
    model::{Segment, TimeRange},
};

/// `range` stretched over every stored Segment that overlaps it, because coalescing may have
/// joined Activity from inside and outside the range into one Segment. Touching Segments do
/// not count: following them would walk the whole day.
#[must_use]
pub fn reach(range: TimeRange, stored: &[Segment]) -> TimeRange {
    stored
        .iter()
        .filter(|s| s.start_at < range.to && s.end_at > range.from)
        .fold(range, |all, s| {
            all.union(TimeRange {
                from: s.start_at,
                to: s.end_at,
            })
        })
}

/// `built` (the Segments for `range`) joined to the stored Segments that touch either edge, when
/// they say the same thing, plus the range to replace so those neighbours are swapped too.
/// Neighbours that do not join are returned unchanged, so the result equals a full rebuild.
#[must_use]
pub fn stitch(
    range: TimeRange,
    built: Vec<Segment>,
    stored: &[Segment],
) -> (TimeRange, Vec<Segment>) {
    let before = stored.iter().filter(|s| s.end_at == range.from);
    let after = stored.iter().filter(|s| s.start_at == range.to);
    let replaced = before.clone().chain(after.clone()).fold(range, |all, s| {
        all.union(TimeRange {
            from: s.start_at,
            to: s.end_at,
        })
    });
    let all = before.cloned().chain(built).chain(after.cloned()).collect();
    (replaced, coalesce(all))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        activity::Source,
        timeline::{model::Away, test_support::at},
    };

    fn seg(start: i64, end: i64, app: &str) -> Segment {
        Segment {
            start_at: at(start),
            end_at: at(end),
            app: app.to_owned(),
            title: String::new(),
            url: None,
            away: Away::No,
            sources: vec![Source::Toggl],
        }
    }

    fn range(from: i64, to: i64) -> TimeRange {
        TimeRange {
            from: at(from),
            to: at(to),
        }
    }

    #[test]
    fn reach_takes_in_a_straddling_segment_but_not_a_touching_one() {
        let stored = [seg(0, 20, "A"), seg(20, 30, "B"), seg(30, 40, "C")];
        assert_eq!(reach(range(10, 30), &stored), range(0, 30));
    }

    #[test]
    fn an_equal_neighbour_on_either_edge_is_joined_and_replaced() {
        let stored = [seg(0, 10, "A"), seg(20, 30, "A")];
        let (replaced, out) = stitch(range(10, 20), vec![seg(10, 20, "A")], &stored);
        assert_eq!(replaced, range(0, 30));
        assert_eq!(out, [seg(0, 30, "A")]);
    }

    #[test]
    fn a_different_neighbour_is_left_as_it_was() {
        let stored = [seg(0, 10, "B")];
        let (_, out) = stitch(range(10, 20), vec![seg(10, 20, "A")], &stored);
        assert_eq!(out, [seg(0, 10, "B"), seg(10, 20, "A")]);
    }

    #[test]
    fn without_neighbours_the_built_segments_pass_through() {
        let (replaced, out) = stitch(range(10, 20), vec![seg(10, 20, "A")], &[]);
        assert_eq!((replaced, out), (range(10, 20), vec![seg(10, 20, "A")]));
    }
}
