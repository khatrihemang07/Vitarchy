//! The Timeline: one account of what happened on this computer, built from every Source's
//! Activity by drawing each trait from the Source that records it best (ADR 0011). Always
//! derived from Activity, so it can be rebuilt at any time.

pub mod away_rules;
#[cfg(test)]
mod away_rules_tests;
mod coalesce;
pub mod domain;
mod merge;
pub mod model;
pub mod no_input;
#[cfg(test)]
mod no_input_tests;
pub mod preview;
#[cfg(test)]
mod preview_tests;
mod rebuild;
mod rules_fingerprint;
mod rules_version;
mod stitch;
mod trim;
pub mod watching;

#[cfg(test)]
mod test_support;

use crate::activity::{Activity, Source};
use model::Segment;

pub use away_rules::AwayRules;
pub use model::TimeRange;
pub use rebuild::{rebuild_all, rebuild_within};
pub use rules_version::ensure_rules_current_or_warn;

/// Builds the Timeline for `activities`: each Source's view of it, trimmed so no Source
/// overlaps itself, merged per trait, then joined where neighbours say the same thing.
#[must_use]
pub fn build(activities: &[Activity]) -> Vec<Segment> {
    let rules = AwayRules::current();
    let observations = Source::all()
        .flat_map(|source| {
            let rows: Vec<&Activity> = activities.iter().filter(|a| a.source == source).collect();
            (source.descriptor().observe)(&rows, &rules)
        })
        .collect();
    let trimmed = trim::trim_self_overlaps(observations);
    coalesce::coalesce(merge::merge(&trimmed, |source: Source| {
        source.descriptor().strengths
    }))
}
