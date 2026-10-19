//! Rung 2: the debit names the other Account by a long number ending in that Account's
//! digits, and the credit of the same amount lands there the same day.

use super::{
    AccountInfo, Candidate, Rung,
    context::{Context, Used},
    found::Found,
    tokens::digit_runs,
};

/// A digit run shorter than this is not an Account number.
const MIN_NUMBER_LEN: usize = 8;
/// Digits shorter than this are too weak to say which Account a number names.
const MIN_KNOWN_DIGITS: usize = 4;

/// Links each unlinked debit to the same-day credit on the Account its number names.
#[must_use]
pub fn find(context: &Context<'_>, used: &Used) -> Found {
    let mut pairs = Vec::new();
    for debit in context.debits.iter().filter(|d| used.debit_free(d.id)) {
        let numbers = long_numbers(debit);
        if numbers.is_empty() {
            continue;
        }
        for credit in context.credits.iter().filter(|c| used.credit_free(c.id)) {
            let Some(account) = context.accounts.get(&credit.account_id) else {
                continue;
            };
            if context.pairable(debit, credit)
                && debit.amount_paise == credit.amount_paise
                && debit.date == credit.date
                && names(&numbers, account)
            {
                pairs.push((debit.id, credit.id));
            }
        }
    }
    Found::from_unique_pairs(&pairs, Rung::AccountNumber)
}

fn long_numbers(debit: &Candidate) -> Vec<&str> {
    debit
        .texts
        .iter()
        .flat_map(|text| digit_runs(text))
        .filter(|run| run.len() >= MIN_NUMBER_LEN)
        .collect()
}

fn names(numbers: &[&str], account: &AccountInfo) -> bool {
    account
        .digits
        .iter()
        .filter(|digits| digits.len() >= MIN_KNOWN_DIGITS)
        .any(|digits| {
            numbers
                .iter()
                .any(|number| number.ends_with(digits.as_str()))
        })
}
