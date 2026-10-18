//! Pairing two sets of rows one-to-one, strongest evidence first, so the order the rows
//! arrive in cannot cause a weak match to steal a row a strong match needed.

use std::cmp::Reverse;

use super::{evidence::Evidence, verdict::verdict};

/// For each row of `left`, the index of the `right` row it is the same Transaction as. Each
/// row pairs at most once, and the result is the same (up to which twin is chosen) when the
/// two sets swap roles.
#[must_use]
pub fn pair(left: &[Evidence], right: &[Evidence]) -> Vec<Option<usize>> {
    let mut candidates = Vec::new();
    for (i, a) in left.iter().enumerate() {
        for (j, b) in right.iter().enumerate() {
            if let Some(found) = verdict(a, b) {
                candidates.push((found.rung, found.days_apart, Reverse(found.closeness), i, j));
            }
        }
    }
    candidates.sort_unstable();
    let mut taken = vec![false; right.len()];
    let mut paired = vec![None; left.len()];
    for (_, _, _, i, j) in candidates {
        if paired[i].is_none() && !taken[j] {
            paired[i] = Some(j);
            taken[j] = true;
        }
    }
    paired
}
