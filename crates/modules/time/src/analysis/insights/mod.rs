//! Insights: plain statements found by simple statistics on the person's own data.
//!
//! Each kind is one file with a pure `find(&Inputs) -> Vec<Insight>`, listed in [`KINDS`].
//! The web writes the sentence from a template per kind. Nothing leaves the machine.

use std::collections::BTreeMap;

mod change;
mod gather;
mod inputs;
mod insight;
mod outlier;
mod score;
mod stats;
mod typical_duration;
mod typical_time;
mod weekday_difference;

pub use gather::{Sources, gather, reach_period, windows};
pub use inputs::{CHANGE_WINDOW_DAYS, DateRange, Inputs, Point, Series};
pub use insight::{Insight, InsightKind, Measure, Numbers, RoutineRef, Subject, SubjectKind};

/// The most Insights one request returns.
pub const MAX_INSIGHTS: usize = 20;

/// The most Insights of one kind in a result, so a kind with many findings (say, a dozen
/// outlying apps) cannot push every other kind off the list.
pub const MAX_PER_KIND: usize = 6;

/// Every Insight kind, in no particular order (the result is ranked by score). To add a kind,
/// write its file and add its `find` here.
const KINDS: &[fn(&Inputs) -> Vec<Insight>] = &[
    typical_time::find,
    typical_duration::find,
    weekday_difference::find,
    change::find,
    outlier::find,
];

/// All Insights the inputs support, most notable first, at most [`MAX_INSIGHTS`] and
/// [`MAX_PER_KIND`] of any kind. Equal scores
/// keep a stable order (by subject, then kind).
#[must_use]
pub fn insights(inputs: &Inputs) -> Vec<Insight> {
    let mut found: Vec<Insight> = KINDS.iter().flat_map(|find| find(inputs)).collect();
    found.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.subject.name.cmp(&b.subject.name))
            .then_with(|| a.subject.routine.cmp(&b.subject.routine))
            .then_with(|| a.kind.cmp(&b.kind))
            .then_with(|| a.measure.cmp(&b.measure))
    });
    let mut per_kind: BTreeMap<InsightKind, usize> = BTreeMap::new();
    found.retain(|insight| {
        let count = per_kind.entry(insight.kind).or_default();
        *count += 1;
        *count <= MAX_PER_KIND
    });
    found.truncate(MAX_INSIGHTS);
    found
}

#[cfg(test)]
mod tests {
    use super::inputs::fixtures::{inputs, series};
    use super::*;

    /// A steady 60 minute length that grows to 90 in the last four weeks: a habit and a change.
    fn growing(name: &str) -> Series {
        let mut s = series(
            Measure::Length,
            &[
                (1, 60.0),
                (2, 60.0),
                (3, 60.0),
                (4, 90.0),
                (5, 90.0),
                (6, 90.0),
            ],
        );
        s.subject.name = name.into();
        s
    }

    #[test]
    fn nothing_measured_gives_no_insights() {
        assert!(insights(&inputs(Vec::new())).is_empty());
    }

    #[test]
    fn the_most_notable_comes_first() {
        let found = insights(&inputs(vec![growing("office")]));
        let kinds: Vec<InsightKind> = found.iter().map(|i| i.kind).collect();
        assert!(kinds.contains(&InsightKind::Change));
        assert_eq!(kinds[0], InsightKind::Change);
        assert!(found.windows(2).all(|w| w[0].score >= w[1].score));
    }

    #[test]
    fn the_list_is_capped() {
        let many = (0..30).map(|i| growing(&format!("step {i:02}"))).collect();
        assert!(insights(&inputs(many)).len() <= MAX_INSIGHTS);
    }

    #[test]
    fn no_kind_crowds_out_the_others() {
        let many = (0..30).map(|i| growing(&format!("step {i:02}"))).collect();
        let found = insights(&inputs(many));
        for kind in [InsightKind::Change, InsightKind::TypicalDuration] {
            assert_eq!(
                found.iter().filter(|i| i.kind == kind).count(),
                MAX_PER_KIND
            );
        }
    }

    #[test]
    fn equal_scores_are_ordered_by_name_whatever_the_input_order() {
        let forward = insights(&inputs(vec![growing("a"), growing("b")]));
        let backward = insights(&inputs(vec![growing("b"), growing("a")]));
        assert_eq!(forward, backward);
        assert_eq!(forward[0].subject.name, "a");
    }
}
