//! Running the rungs strongest first, each on what the earlier ones left.

use super::{
    Detection, Input, Link, account_number, bill_payment,
    context::{Context, Used},
    found::Found,
    reference, unique_amount,
};

/// The automatic rungs, strongest first. Variation is data: a new rung is one more entry.
const RUNGS: &[fn(&Context<'_>, &Used) -> Found] = &[
    reference::find,
    account_number::find,
    bill_payment::find,
    unique_amount::find,
];

/// Finds the Transfers `input` implies and are not already stored.
#[must_use]
pub fn detect(input: &Input<'_>) -> Detection {
    let context = Context::new(input);
    let mut used = Used::of(input.transfers);
    let mut detection = Detection::default();
    for find in RUNGS {
        let found = find(&context, &used);
        detection.ambiguous += found.ambiguous;
        for link in found
            .links
            .into_iter()
            .filter(|l| in_focus(input, l, &context))
        {
            used.take(&link);
            detection.found.add(link.rung);
            detection.links.push(link);
        }
    }
    detection
}

fn in_focus(input: &Input<'_>, link: &Link, context: &Context<'_>) -> bool {
    let Some((from, to)) = input.focus else {
        return true;
    };
    context
        .debits
        .iter()
        .find(|d| d.id == link.debit_id)
        .is_some_and(|d| (from..=to).contains(&d.date))
}
