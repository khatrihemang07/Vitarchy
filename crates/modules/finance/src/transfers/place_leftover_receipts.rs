//! Pass 2 of the bill-payment rung: a receipt left over joins the one payment that still has
//! room for it, making (or growing) a partial Transfer. Where two payments have room, or
//! several receipts together would overfill the one that does, nothing is linked.

use std::collections::BTreeMap;

use super::{
    BILL_TOLERANCE_PAISE, Candidate,
    assignment::{Assigned, Assignment},
    bill_payment::{Payment, in_reach},
    context::Context,
};

/// Assigns each receipt to the only payment it fits, when its group does not overfill it.
#[must_use]
pub fn assign(
    context: &Context<'_>,
    payments: &[Payment<'_>],
    receipts: &[&Candidate],
) -> Assigned {
    let mut by_payment: BTreeMap<i64, Vec<&Candidate>> = BTreeMap::new();
    let mut ambiguous = 0;
    for receipt in receipts {
        let fits: Vec<&Payment<'_>> = payments
            .iter()
            .filter(|p| in_reach(context, p.debit, receipt) && has_room_for(p, receipt))
            .collect();
        match fits.as_slice() {
            [] => {}
            [only] => by_payment.entry(only.debit.id).or_default().push(receipt),
            _ => ambiguous += 1,
        }
    }
    let mut assigned = Vec::new();
    for (debit_id, group) in by_payment {
        let payment = payments.iter().find(|p| p.debit.id == debit_id);
        let total: i64 = group.iter().map(|r| r.amount_paise).sum();
        if payment
            .is_some_and(|p| p.linked_paise + total <= p.debit.amount_paise + BILL_TOLERANCE_PAISE)
        {
            assigned.push(Assignment {
                debit_id,
                receipt_ids: group.iter().map(|r| r.id).collect(),
            });
        } else {
            ambiguous += group.len();
        }
    }
    Assigned {
        assigned,
        ambiguous,
    }
}

/// A payment has room while it is not yet covered and the receipt would not take it past
/// the tolerance.
fn has_room_for(payment: &Payment<'_>, receipt: &Candidate) -> bool {
    payment.linked_paise < payment.debit.amount_paise
        && payment.linked_paise + receipt.amount_paise
            <= payment.debit.amount_paise + BILL_TOLERANCE_PAISE
}
