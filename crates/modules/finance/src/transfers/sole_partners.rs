//! Keeping only the pairs where each side has no other partner.

use std::collections::HashMap;

/// The (debit, credit) pairs in which neither has another candidate partner in `pairs`, and
/// how many debits were left out because they had several (or shared their credit).
#[must_use]
pub fn mutual(pairs: &[(i64, i64)]) -> (Vec<(i64, i64)>, usize) {
    let mut debit_degree: HashMap<i64, usize> = HashMap::new();
    let mut credit_degree: HashMap<i64, usize> = HashMap::new();
    for (debit, credit) in pairs {
        *debit_degree.entry(*debit).or_default() += 1;
        *credit_degree.entry(*credit).or_default() += 1;
    }
    let alone =
        |(debit, credit): &&(i64, i64)| debit_degree[debit] == 1 && credit_degree[credit] == 1;
    let kept: Vec<(i64, i64)> = pairs.iter().filter(alone).copied().collect();
    let ambiguous = debit_degree.len() - kept.len();
    (kept, ambiguous)
}

#[cfg(test)]
mod tests {
    use super::mutual;

    #[test]
    fn a_lone_pair_is_kept() {
        assert_eq!(mutual(&[(1, 2)]), (vec![(1, 2)], 0));
    }

    #[test]
    fn a_debit_with_two_credits_is_ambiguous() {
        assert_eq!(mutual(&[(1, 2), (1, 3)]), (vec![], 1));
    }

    #[test]
    fn two_debits_for_one_credit_are_both_ambiguous() {
        assert_eq!(mutual(&[(1, 3), (2, 3)]), (vec![], 2));
    }

    #[test]
    fn other_pairs_are_unaffected_by_an_ambiguous_one() {
        assert_eq!(mutual(&[(1, 2), (3, 4), (3, 5)]), (vec![(1, 2)], 1));
    }
}
