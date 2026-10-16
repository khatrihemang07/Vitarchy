//! Drops near-duplicate chains: a chain and a longer one that contains it say the same thing.

use std::collections::{HashMap, HashSet};

/// A longer chain replaces the shorter one inside it when it happens on at least this percent of
/// the shorter one's Days; if it is rarer, the shorter one stays and the longer is dropped.
const KEEP_LONGER_PERCENT: usize = 80;

/// `chains` with their Days, minus every chain that is a contiguous part of a longer one it is
/// nearly as frequent as, and minus every longer chain that is much rarer than a part of it.
/// Chains that merely overlap are not related, and both stay.
pub(super) fn drop_redundant(chains: Vec<(Vec<&str>, usize)>) -> Vec<(Vec<&str>, usize)> {
    let keep: Vec<bool> = {
        let days: HashMap<&[&str], usize> = chains
            .iter()
            .map(|(chain, days)| (chain.as_slice(), *days))
            .collect();
        let mut dropped: HashSet<&[&str]> = HashSet::new();
        for (long, long_days) in &chains {
            for len in 2..long.len() {
                for part in long.windows(len) {
                    let Some(part_days) = days.get(part) else {
                        continue;
                    };
                    if long_days * 100 >= part_days * KEEP_LONGER_PERCENT {
                        dropped.insert(part);
                    } else {
                        dropped.insert(long.as_slice());
                    }
                }
            }
        }
        chains
            .iter()
            .map(|(chain, _)| !dropped.contains(chain.as_slice()))
            .collect()
    };
    chains
        .into_iter()
        .zip(keep)
        .filter_map(|(chain, keep)| keep.then_some(chain))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kept(chains: &[(&[&'static str], usize)]) -> Vec<Vec<&'static str>> {
        let input = chains.iter().map(|(c, d)| (c.to_vec(), *d)).collect();
        let mut out: Vec<_> = drop_redundant(input).into_iter().map(|(c, _)| c).collect();
        out.sort();
        out
    }

    #[test]
    fn the_longer_chain_wins_when_nearly_as_frequent() {
        assert_eq!(
            kept(&[(&["a", "b"], 10), (&["a", "b", "c"], 8)]),
            [vec!["a", "b", "c"]]
        );
    }

    #[test]
    fn the_shorter_chain_wins_when_the_longer_is_much_rarer() {
        assert_eq!(
            kept(&[(&["a", "b"], 10), (&["a", "b", "c"], 7)]),
            [vec!["a", "b"]]
        );
    }

    #[test]
    fn overlapping_chains_that_do_not_contain_each_other_both_stay() {
        assert_eq!(
            kept(&[(&["a", "b"], 10), (&["b", "c"], 10)]),
            [vec!["a", "b"], vec!["b", "c"]]
        );
    }

    #[test]
    fn a_part_with_a_gap_is_not_a_contiguous_part() {
        assert_eq!(
            kept(&[(&["a", "c"], 10), (&["a", "b", "c"], 10)]),
            [vec!["a", "b", "c"], vec!["a", "c"]]
        );
    }
}
