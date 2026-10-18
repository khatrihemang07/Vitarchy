//! Pairing two sets of rows one-to-one, strongest evidence first, so the order the rows
//! arrive in cannot cause a weak match to steal a row a strong match needed. The unique
//! amount-and-date rung comes last and needs a whole-set view, so it runs after the rest.

use std::cmp::Reverse;

use super::{
    evidence::Evidence,
    verdict::{unique_partner_possible, verdict},
};

/// A stored Transaction as pairing sees it: what its resolved view shows, then what each of its
/// Printings printed. It is the same as a row when any of them is, so a payment app's id
/// still identifies a Transaction whose shown id came from a bank.
pub type Forms<'a> = Vec<&'a Evidence>;

/// [`pair_forms`] for rows that each have one form.
#[cfg(test)]
#[must_use]
pub fn pair_evidence(left: &[Evidence], right: &[Evidence]) -> Vec<Option<usize>> {
    let forms: Vec<Forms<'_>> = right.iter().map(|row| vec![row]).collect();
    pair_forms(left, &forms)
}

/// For each row of `left`, the index of the `right` row it is the same Transaction as. Each
/// row pairs at most once, and the result is the same (up to which twin is chosen) when the
/// two sets swap roles. The first form of each `right` row is its resolved view.
#[must_use]
pub fn pair_forms(left: &[Evidence], right: &[Forms<'_>]) -> Vec<Option<usize>> {
    let mut candidates = Vec::new();
    for (i, a) in left.iter().enumerate() {
        for (j, forms) in right.iter().enumerate() {
            let best = forms
                .iter()
                .filter_map(|b| verdict(a, b))
                .min_by_key(|v| (v.rung, v.days_apart, Reverse(v.closeness)));
            if let Some(found) = best {
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
    let resolved: Vec<&Evidence> = right.iter().map(|forms| forms[0]).collect();
    pair_by_unique_partner(left, &resolved, &mut paired, &mut taken);
    paired
}

/// The last rung: an unpaired row joins an unpaired row of the other kind of file when each
/// is the other's only possible partner. Every row counts as a possible partner, paired or
/// not, so a row that stronger evidence already claimed still makes the others ambiguous.
fn pair_by_unique_partner(
    left: &[Evidence],
    right: &[&Evidence],
    paired: &mut [Option<usize>],
    taken: &mut [bool],
) {
    for (i, a) in left.iter().enumerate() {
        if paired[i].is_some() {
            continue;
        }
        let mut partners = (0..right.len()).filter(|&j| unique_partner_possible(a, right[j]));
        let (Some(j), None) = (partners.next(), partners.next()) else {
            continue;
        };
        let back = left.iter().filter(|b| unique_partner_possible(b, right[j]));
        if !taken[j] && back.count() == 1 {
            paired[i] = Some(j);
            taken[j] = true;
        }
    }
}
