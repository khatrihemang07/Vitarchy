//! Rung 1: the debit's reference appears as a whole token in the credit's description or is
//! the credit's own reference, on the same amount within a day.

use super::{
    Candidate, Rung,
    context::{Context, Used, days_apart},
    found::Found,
    tokens::words,
};

/// A reference shorter than this is too common to be evidence.
const MIN_REFERENCE_LEN: usize = 8;

/// Links each unlinked debit to the one credit that carries its reference.
#[must_use]
pub fn find(context: &Context<'_>, used: &Used) -> Found {
    let mut pairs = Vec::new();
    for debit in context.debits.iter().filter(|d| used.debit_free(d.id)) {
        let keys = reference_keys(debit);
        if keys.is_empty() {
            continue;
        }
        for credit in context.credits.iter().filter(|c| used.credit_free(c.id)) {
            if context.pairable(debit, credit)
                && debit.amount_paise == credit.amount_paise
                && days_apart(debit.date, credit.date) <= 1
                && mentions(credit, &keys)
            {
                pairs.push((debit.id, credit.id));
            }
        }
    }
    Found::from_unique_pairs(&pairs, Rung::Reference)
}

fn reference_keys(debit: &Candidate) -> Vec<String> {
    debit
        .references
        .iter()
        .map(|r| r.trim().to_uppercase())
        .filter(|r| r.len() >= MIN_REFERENCE_LEN)
        .collect()
}

fn mentions(credit: &Candidate, keys: &[String]) -> bool {
    let own = credit.references.iter().map(|r| r.trim().to_uppercase());
    let printed = credit.texts.iter().flat_map(|text| words(text));
    own.chain(printed).any(|word| keys.contains(&word))
}
