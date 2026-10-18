//! The merge engine: one account of what happened, drawn from every Source's observations.
//! Pure; it knows nothing about any particular Source, only the strengths it is handed.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};

use super::model::{Away, Observation, Segment};
use crate::activity::{Source, Strengths};

/// One Source's observations, in time order, with a cursor for the sweep.
struct Lane<'a> {
    strengths: Strengths,
    items: Vec<&'a Observation>,
    next: usize,
}

impl<'a> Lane<'a> {
    /// The observation that covers all of `from..to`, if any. Edges include every observation's
    /// ends, so an observation either covers an elementary interval fully or not at all.
    fn covering(&mut self, from: DateTime<Utc>, to: DateTime<Utc>) -> Option<Cover<'a>> {
        while self.items.get(self.next).is_some_and(|o| o.end_at <= from) {
            self.next += 1;
        }
        let observation = *self.items.get(self.next)?;
        (observation.start_at <= from && observation.end_at >= to).then_some(Cover {
            source: observation.source,
            strengths: self.strengths,
            observation,
        })
    }
}

/// A Source's say over one elementary interval.
struct Cover<'a> {
    source: Source,
    strengths: Strengths,
    observation: &'a Observation,
}

/// Splits time at every observation edge and, for each stretch something covers, draws each
/// trait from the highest-ranked covering Source that has it (ties go to the Source listed
/// first by `Source::all()`). Stretches nothing covers stay gaps. Each Source's own
/// observations must not overlap (see `trim_self_overlaps`). Output is in time order and
/// not yet coalesced.
#[must_use]
pub fn merge(
    observations: &[Observation],
    strengths: impl Fn(Source) -> Strengths,
) -> Vec<Segment> {
    let mut lanes: Vec<Lane> = Source::all()
        .map(|source| {
            let mut items: Vec<&Observation> =
                observations.iter().filter(|o| o.source == source).collect();
            items.sort_by_key(|o| o.start_at);
            Lane {
                strengths: strengths(source),
                items,
                next: 0,
            }
        })
        .collect();
    let edges: Vec<DateTime<Utc>> = observations
        .iter()
        .flat_map(|o| [o.start_at, o.end_at])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    edges
        .windows(2)
        .filter_map(|pair| {
            let covering: Vec<Cover> = lanes
                .iter_mut()
                .filter_map(|lane| lane.covering(pair[0], pair[1]))
                .collect();
            segment_for(pair[0], pair[1], &covering)
        })
        .collect()
}

/// The highest-ranked covering observation for a trait; the first listed wins a tie.
fn best<'a>(
    covering: &[Cover<'a>],
    rank: impl Fn(&Strengths) -> Option<u8>,
    usable: impl Fn(&Observation) -> bool,
) -> Option<&'a Observation> {
    covering
        .iter()
        .filter(|c| usable(c.observation))
        .filter_map(|c| rank(&c.strengths).map(|r| (r, c.observation)))
        .fold(
            None,
            |top: Option<(u8, &Observation)>, candidate| match top {
                Some((rank, _)) if rank >= candidate.0 => top,
                _ => Some(candidate),
            },
        )
        .map(|(_, observation)| observation)
}

fn segment_for(from: DateTime<Utc>, to: DateTime<Utc>, covering: &[Cover]) -> Option<Segment> {
    // A stretch no Source ranks for boundaries still shows what the first covering Source saw.
    let shape = best(covering, |s| s.boundaries, |_| true)
        .or_else(|| covering.first().map(|c| c.observation))?;
    // A page address belongs to the app it was seen in, so it is only taken from an
    // observation of the app that won the boundaries.
    let url = best(covering, |s| s.url, |o| o.app == shape.app).and_then(|o| o.url.clone());
    let away = match best(covering, |s| s.away, |o| o.away.is_some()).and_then(|o| o.away) {
        Some(true) => Away::Yes,
        Some(false) => Away::No,
        None => Away::Unknown,
    };
    Some(Segment {
        start_at: from,
        end_at: to,
        app: shape.app.clone(),
        title: shape.title.clone(),
        url,
        away,
        sources: Source::all()
            .filter(|s| covering.iter().any(|c| c.source == *s))
            .collect(),
    })
}

#[cfg(test)]
mod tests;
