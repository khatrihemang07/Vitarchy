//! The two ways a Payment record finds its Transaction: the bank reference (UTR), then the
//! same amount and direction on a nearby date.

use super::facts::{RecordFacts, TransactionFacts};
use crate::accounts::{same_issuer, same_number};

/// How many days apart a record and its Transaction may be when matched by amount.
const DATE_WINDOW_DAYS: i64 = 1;

pub(super) fn utr(record: &RecordFacts) -> Option<&str> {
    record
        .utr
        .as_deref()
        .map(str::trim)
        .filter(|utr| !utr.is_empty())
}

/// Whether the record says which instrument paid; without that, an amount and a date are
/// too weak to pick an Account, so only the UTR may link it.
pub(super) fn names_instrument(record: &RecordFacts) -> bool {
    record.instrument_issuer.is_some() || record.instrument_digits.is_some()
}

/// Whether the Transaction's Account agrees with the record's instrument on every detail
/// the record gives.
fn account_agrees(record: &RecordFacts, tx: &TransactionFacts) -> bool {
    let issuer_ok = record
        .instrument_issuer
        .as_deref()
        .is_none_or(|issuer| same_issuer(issuer, &tx.account_issuer));
    let digits_ok = record.instrument_digits.as_deref().is_none_or(|digits| {
        tx.account_mask
            .as_deref()
            .is_some_and(|mask| same_number(digits, mask))
    });
    issuer_ok && digits_ok
}

/// A refund shares its UTR with the original payment, so direction must agree too.
pub(super) fn by_utr(
    record: &RecordFacts,
    transactions: &[TransactionFacts],
    taken: &[bool],
) -> Option<usize> {
    let utr = utr(record)?;
    transactions.iter().enumerate().position(|(index, tx)| {
        !taken[index]
            && account_agrees(record, tx)
            && tx.direction == record.direction
            && (tx.reference.as_deref().map(str::trim) == Some(utr)
                || mentions(&tx.description, utr))
    })
}

/// The nearest-dated free Transaction of the same amount and direction. Two different
/// references mean two different payments, so those never pair.
pub(super) fn by_amount_and_date(
    record: &RecordFacts,
    transactions: &[TransactionFacts],
    taken: &[bool],
) -> Option<usize> {
    let record_utr = utr(record);
    transactions
        .iter()
        .enumerate()
        .filter(|(index, tx)| {
            !taken[*index]
                && account_agrees(record, tx)
                && tx.amount_paise == record.amount_paise
                && tx.direction == record.direction
                && (tx.date - record.on).num_days().abs() <= DATE_WINDOW_DAYS
                && !references_differ(record_utr, tx.reference.as_deref())
        })
        .min_by_key(|(_, tx)| (tx.date - record.on).num_days().abs())
        .map(|(index, _)| index)
}

fn references_differ(utr: Option<&str>, reference: Option<&str>) -> bool {
    matches!((utr, reference.map(str::trim)), (Some(a), Some(b)) if !b.is_empty() && a != b)
}

/// Whether `description` holds `utr` as a whole digit run: a longer number that merely
/// contains it (an amount, another reference) is not the same reference.
fn mentions(description: &str, utr: &str) -> bool {
    description.match_indices(utr).any(|(start, found)| {
        let before = description[..start].chars().next_back();
        let after = description[start + found.len()..].chars().next();
        !before.is_some_and(|c| c.is_ascii_digit()) && !after.is_some_and(|c| c.is_ascii_digit())
    })
}
