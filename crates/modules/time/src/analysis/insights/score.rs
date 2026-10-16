//! How notable an Insight is.
//!
//! `score = base(kind) + SPAN * effect * confidence`
//!
//! - `base` ranks the kinds: a departure from the usual (a change, an outlier, a weekday
//!   difference) outranks a steady habit, since it is news.
//! - `effect`, 0 to 1, is how strong the finding is for its kind (a tight habit, a big gap).
//! - `confidence` is the Days measured against [`FULL_CONFIDENCE_DAYS`], capped at 1.
//!
//! Scores run from 20 to 90 and are only used to order Insights.

use super::insight::InsightKind;

/// What each kind scores before its strength and sample size count.
const BASE_SCORES: [(InsightKind, f64); 5] = [
    (InsightKind::Change, 50.0),
    (InsightKind::Outlier, 45.0),
    (InsightKind::WeekdayDifference, 40.0),
    (InsightKind::TypicalTime, 30.0),
    (InsightKind::TypicalDuration, 20.0),
];

/// The most a strong, well-measured finding adds to its kind's base.
const SPAN: f64 = 40.0;

/// Days after which more samples add no confidence.
const FULL_CONFIDENCE_DAYS: f64 = 20.0;

/// The notability of a finding of `kind` with strength `effect` (clamped to 0 to 1) over `samples` Days.
#[must_use]
pub fn score(kind: InsightKind, effect: f64, samples: usize) -> f64 {
    let base = BASE_SCORES
        .iter()
        .find(|(known, _)| *known == kind)
        .map_or(0.0, |(_, base)| *base);
    // Sample counts are tiny next to f64's exact range.
    #[allow(clippy::cast_precision_loss)]
    let confidence = (samples as f64 / FULL_CONFIDENCE_DAYS).min(1.0);
    base + SPAN * effect.clamp(0.0, 1.0) * confidence
}

/// How strong a gap of `minutes` is, reaching 1 at `full_at` minutes.
#[must_use]
pub fn gap_effect(minutes: f64, full_at: f64) -> f64 {
    (minutes.abs() / full_at).min(1.0)
}

#[cfg(test)]
// The expected figures are exact by construction (small whole minutes).
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn a_strong_well_measured_change_scores_highest() {
        assert_eq!(score(InsightKind::Change, 1.0, 20), 90.0);
    }

    #[test]
    fn strength_and_sample_size_both_scale_the_bonus() {
        assert_eq!(score(InsightKind::Change, 0.5, 10), 60.0);
        assert_eq!(score(InsightKind::TypicalDuration, 0.0, 50), 20.0);
    }

    #[test]
    fn samples_beyond_full_confidence_and_effects_beyond_one_add_nothing() {
        assert_eq!(score(InsightKind::Outlier, 3.0, 200), 85.0);
        assert_eq!(score(InsightKind::Outlier, -1.0, 5), 45.0);
    }

    #[test]
    fn every_kind_has_a_base() {
        for (kind, base) in BASE_SCORES {
            assert_eq!(score(kind, 0.0, 1), base);
        }
        assert_eq!(BASE_SCORES.len(), 5);
    }
}
