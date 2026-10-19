//! Which few receipts could together be one bill payment.

/// The most receipts one bill payment is searched across.
pub const MAX_RECEIPTS: usize = 4;
/// The most receipts considered at once; a window holding more is too crowded to judge.
pub const MAX_POOL: usize = 24;

/// Index sets of 1 to [`MAX_RECEIPTS`] of `amounts` that add up to at least `target` and at
/// most `target + tolerance`, each in ascending order.
#[must_use]
pub fn fitting(amounts: &[i64], target: i64, tolerance: i64) -> Vec<Vec<usize>> {
    let mut found = Vec::new();
    if amounts.len() <= MAX_POOL {
        grow(amounts, target, tolerance, &mut Vec::new(), 0, &mut found);
    }
    found
}

fn grow(
    amounts: &[i64],
    target: i64,
    tolerance: i64,
    chosen: &mut Vec<usize>,
    from: usize,
    found: &mut Vec<Vec<usize>>,
) {
    let sum: i64 = chosen.iter().map(|&i| amounts[i]).sum();
    if !chosen.is_empty() && (target..=target + tolerance).contains(&sum) {
        found.push(chosen.clone());
    }
    if chosen.len() == MAX_RECEIPTS || sum > target + tolerance {
        return;
    }
    for next in from..amounts.len() {
        chosen.push(next);
        grow(amounts, target, tolerance, chosen, next + 1, found);
        chosen.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_and_split_payments_are_found() {
        assert_eq!(fitting(&[600, 400, 50], 1_000, 0), vec![vec![0, 1]]);
        assert_eq!(fitting(&[1_000, 7], 1_000, 0), vec![vec![0]]);
    }

    #[test]
    fn the_tolerance_allows_a_little_more_but_never_less() {
        assert_eq!(fitting(&[1_005], 1_000, 50), vec![vec![0]]);
        assert!(fitting(&[1_051], 1_000, 50).is_empty());
        assert!(fitting(&[999], 1_000, 50).is_empty());
    }

    #[test]
    fn at_most_four_receipts_are_combined() {
        assert!(fitting(&[200; 5], 1_000, 0).is_empty());
        assert_eq!(fitting(&[250; 4], 1_000, 0), vec![vec![0, 1, 2, 3]]);
    }

    #[test]
    fn every_fitting_set_is_reported() {
        assert_eq!(fitting(&[100, 100], 100, 0), vec![vec![0], vec![1]]);
    }

    #[test]
    fn a_crowded_pool_is_not_judged() {
        assert!(fitting(&[1; 25], 1, 0).is_empty());
    }
}
