//! The Timeline: one account of what happened on this computer, built from every Source's
//! Activity by drawing each trait from the Source that records it best (ADR 0011). Always
//! derived from Activity, so it can be rebuilt at any time.

mod coalesce;
mod merge;
pub mod model;
mod rebuild;
mod stitch;
mod trim;

#[cfg(test)]
mod test_support;

use crate::activity::{Activity, Source};
use model::Segment;

pub use model::TimeRange;
pub use rebuild::{rebuild_all, rebuild_within};

/// Builds the Timeline for `activities`: each Source's view of it, trimmed so no Source
/// overlaps itself, merged per trait, then joined where neighbours say the same thing.
#[must_use]
pub fn build(activities: &[Activity]) -> Vec<Segment> {
    let observations = activities
        .iter()
        .flat_map(|activity| (activity.source.descriptor().observe)(activity))
        .collect();
    let trimmed = trim::trim_self_overlaps(observations);
    coalesce::coalesce(merge::merge(&trimmed, |source: Source| {
        source.descriptor().strengths
    }))
}
