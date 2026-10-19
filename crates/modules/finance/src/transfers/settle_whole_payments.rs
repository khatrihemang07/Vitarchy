//! Pass 1 of the bill-payment rung: payments whose receipts are all there. A set of up to
//! four receipts fits when it adds up to the payment, or to at most ₹50 more. An exact set
//! beats one with extra. A payment is assigned only when one set fits it and no other
//! payment could claim any of the same receipts on equal or better terms; assigning one
//! payment can settle the next, so the pass repeats until nothing more is decided.

use std::collections::HashSet;

use super::{
    BILL_TOLERANCE_PAISE, Candidate,
    assignment::{Assigned, Assignment},
    bill_payment::in_reach,
    context::Context,
    subsets::fitting,
};

/// The sets of receipts that could settle one payment: all exact, or else all with extra.
struct Claims<'a> {
    debit: &'a Candidate,
    exact: bool,
    sets: Vec<Vec<i64>>,
}

/// Assigns receipts to the payments (all without a Transfer) that one set of them settles.
#[must_use]
pub fn assign(context: &Context<'_>, payments: &[&Candidate], receipts: &[&Candidate]) -> Assigned {
    let mut taken: HashSet<i64> = HashSet::new();
    let mut open: Vec<&Candidate> = payments.to_vec();
    let mut assigned = Vec::new();
    loop {
        let claims: Vec<Claims<'_>> = open
            .iter()
            .filter_map(|d| claims_of(context, d, receipts, &taken))
            .collect();
        let winners: Vec<&Claims<'_>> = claims.iter().filter(|c| wins(c, &claims)).collect();
        if winners.is_empty() {
            return Assigned {
                assigned,
                ambiguous: claims.len(),
            };
        }
        for win in winners {
            taken.extend(&win.sets[0]);
            open.retain(|d| d.id != win.debit.id);
            assigned.push(Assignment {
                debit_id: win.debit.id,
                receipt_ids: win.sets[0].clone(),
            });
        }
    }
}

fn claims_of<'a>(
    context: &Context<'_>,
    debit: &'a Candidate,
    receipts: &[&Candidate],
    taken: &HashSet<i64>,
) -> Option<Claims<'a>> {
    let pool: Vec<&Candidate> = receipts
        .iter()
        .copied()
        .filter(|r| !taken.contains(&r.id) && in_reach(context, debit, r))
        .collect();
    let amounts: Vec<i64> = pool.iter().map(|r| r.amount_paise).collect();
    let fits = fitting(&amounts, debit.amount_paise, BILL_TOLERANCE_PAISE);
    let sum = |set: &[usize]| set.iter().map(|&i| amounts[i]).sum::<i64>();
    let exact: Vec<&Vec<usize>> = fits
        .iter()
        .filter(|s| sum(s) == debit.amount_paise)
        .collect();
    let (chosen, is_exact) = if exact.is_empty() {
        (fits.iter().collect::<Vec<_>>(), false)
    } else {
        (exact, true)
    };
    if chosen.is_empty() {
        return None;
    }
    let sets = chosen
        .into_iter()
        .map(|set| set.iter().map(|&i| pool[i].id).collect())
        .collect();
    Some(Claims {
        debit,
        exact: is_exact,
        sets,
    })
}

/// A claim wins when one set fits and every other payment that could take any of the same
/// receipts is on worse terms (extra, where this one is exact).
fn wins(claim: &Claims<'_>, all: &[Claims<'_>]) -> bool {
    if claim.sets.len() != 1 {
        return false;
    }
    let mine: HashSet<i64> = claim.sets[0].iter().copied().collect();
    all.iter()
        .filter(|other| other.debit.id != claim.debit.id)
        .filter(|other| other.sets.iter().flatten().any(|id| mine.contains(id)))
        .all(|other| claim.exact && !other.exact)
}
