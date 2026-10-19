//! Rung 4: the debit and the credit are each other's only candidate: the same amount within
//! a day, on different Accounts, with no rival on either side.

use std::collections::HashSet;

use super::{
    Rung,
    context::{Context, Used, days_apart},
    found::{Found, pairs_to_links},
    sole_partners::mutual,
};

/// Links the debits and credits that are each other's only candidate. Rivals count even when
/// an earlier rung already linked them: a wrong guess costs more than a missed one.
#[must_use]
pub fn find(context: &Context<'_>, used: &Used) -> Found {
    let mut pairs = Vec::new();
    for debit in &context.debits {
        for credit in &context.credits {
            if context.pairable(debit, credit)
                && debit.amount_paise == credit.amount_paise
                && days_apart(debit.date, credit.date) <= 1
            {
                pairs.push((debit.id, credit.id));
            }
        }
    }
    let (kept, _) = mutual(&pairs);
    let linked: HashSet<i64> = kept.iter().map(|(debit, _)| *debit).collect();
    let ambiguous: HashSet<i64> = pairs
        .iter()
        .map(|(debit, _)| *debit)
        .filter(|debit| used.debit_free(*debit) && !linked.contains(debit))
        .collect();
    let free = kept
        .into_iter()
        .filter(|(debit, credit)| used.debit_free(*debit) && used.credit_free(*credit))
        .collect();
    Found {
        links: pairs_to_links(free, Rung::UniqueAmount),
        ambiguous: ambiguous.len(),
    }
}
