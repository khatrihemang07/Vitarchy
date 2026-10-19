//! Rung 3: a card-bill payment and the receipts on cards it paid. Receipts may add up to a
//! little more than the payment (cashback), and a payment may be split across cards, some of
//! whose Statements may not be imported yet.
//!
//! Two passes: first payments whose receipts are all there (see [`super::settle_whole_payments`]),
//! then receipts still unassigned that fit the one payment with room (see
//! [`super::place_leftover_receipts`]).

use super::{
    Candidate, Link, Rung,
    assignment::Assignment,
    context::{Context, Used, days_apart},
    found::Found,
    place_leftover_receipts, settle_whole_payments,
};
use crate::reader::contract::AccountKind;

/// A card-bill payment and what is already linked to it.
pub struct Payment<'a> {
    /// The Money-out Transaction.
    pub debit: &'a Candidate,
    /// What its stored Transfer holds so far; 0 when it has none yet.
    pub linked_paise: i64,
}

/// Links card-bill payments to their receipts.
#[must_use]
pub fn find(context: &Context<'_>, used: &Used) -> Found {
    let receipts = open_receipts(context, used);
    let payments = payments(context, used);
    let fresh: Vec<&Candidate> = payments
        .iter()
        .filter(|p| p.linked_paise == 0)
        .map(|p| p.debit)
        .collect();
    let whole = settle_whole_payments::assign(context, &fresh, &receipts);
    let left: Vec<&Candidate> = receipts
        .iter()
        .copied()
        .filter(|r| !whole.assigned.iter().any(|a| a.receipt_ids.contains(&r.id)))
        .collect();
    let open: Vec<Payment<'_>> = payments
        .into_iter()
        .filter(|p| !whole.assigned.iter().any(|a| a.debit_id == p.debit.id))
        .collect();
    let leftover = place_leftover_receipts::assign(context, &open, &left);
    Found {
        ambiguous: whole.ambiguous + leftover.ambiguous,
        links: links_of(whole.assigned, leftover.assigned),
    }
}

/// The card credits that read as payment receipts and are in no Transfer yet.
fn open_receipts<'a>(context: &Context<'a>, used: &Used) -> Vec<&'a Candidate> {
    context
        .credits
        .iter()
        .copied()
        .filter(|c| used.credit_free(c.id) && is_receipt(context, c))
        .collect()
}

/// Both passes' assignments as links, ordered by debit.
fn links_of(whole: Vec<Assignment>, leftover: Vec<Assignment>) -> Vec<Link> {
    let mut links: Vec<Link> = whole
        .into_iter()
        .chain(leftover)
        .map(|a| Link {
            debit_id: a.debit_id,
            credit_ids: a.receipt_ids,
            rung: Rung::BillPayment,
        })
        .collect();
    links.sort_by_key(|l| l.debit_id);
    links
}

/// Whether the receipt may belong to the payment: other Account, within a day.
#[must_use]
pub fn in_reach(context: &Context<'_>, debit: &Candidate, receipt: &Candidate) -> bool {
    context.pairable(debit, receipt) && days_apart(debit.date, receipt.date) <= 1
}

fn is_receipt(context: &Context<'_>, credit: &Candidate) -> bool {
    let on_card = context
        .accounts
        .get(&credit.account_id)
        .is_some_and(|a| a.kind == AccountKind::CreditCard);
    on_card && context.markers.is_receipt(&credit.texts)
}

/// Card-bill payments that can still take receipts: those with no Transfer yet and those
/// whose bill-payment Transfer is stored.
fn payments<'a>(context: &Context<'a>, used: &Used) -> Vec<Payment<'a>> {
    context
        .debits
        .iter()
        .copied()
        .filter(|d| context.markers.is_payee(&d.texts))
        .filter_map(|debit| {
            if used.debit_free(debit.id) {
                return Some(Payment {
                    debit,
                    linked_paise: 0,
                });
            }
            let stored = context.transfers.get(&debit.id)?;
            (stored.rung == Rung::BillPayment).then_some(Payment {
                debit,
                linked_paise: stored.linked_paise,
            })
        })
        .collect()
}
