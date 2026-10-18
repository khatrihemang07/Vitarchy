//! Matching a payment app's instrument (how it names the Account it paid from, such as
//! `XXXXXX4321` or `ICICI R`) to an Account. An instrument does not say what kind of Account
//! it is, so it is matched across kinds, by what the person confirmed before and then by the
//! issuer and visible digits it shows.

use std::{collections::HashMap, hash::BuildHasher};

use super::{AccountIdentity, Match, candidates, mask::remembered_digits, matched};
use crate::reader::contract::ProposedAccount;

/// What remembers an instrument: its text with runs of whitespace squashed and upper-cased,
/// so `icici  r` and `ICICI R` are one instrument. `None` for a blank or missing one.
#[must_use]
pub fn instrument_key(instrument: Option<&str>) -> Option<String> {
    let squashed = instrument?.split_whitespace().collect::<Vec<_>>().join(" ");
    (!squashed.is_empty()).then(|| squashed.to_uppercase())
}

/// The Account an instrument is: the remembered one if the person confirmed it before, else
/// the Accounts that show the same visible digits (and the issuer, when the instrument names
/// one), else, when the instrument shows no digits, the Accounts of its issuer. A match made
/// by issuer alone is [`Match::OneWithoutNumber`] when it is a single Account.
#[must_use]
pub fn find_instrument<S: BuildHasher>(
    proposal: &ProposedAccount,
    remembered: &HashMap<String, i64, S>,
    identities: &[AccountIdentity],
) -> Match {
    let key = instrument_key(proposal.mask.as_deref());
    let known_account = |id: &i64| identities.iter().any(|entry| entry.account_id == *id);
    if let Some(id) = key
        .as_ref()
        .and_then(|key| remembered.get(key))
        .filter(|id| known_account(id))
    {
        return Match::One(*id);
    }
    let issuer = proposal.issuer.as_deref();
    let printed = remembered_digits(proposal.mask.as_deref()).and(proposal.mask.as_deref());
    if printed.is_none() && issuer.is_none() {
        return Match::None;
    }
    matched(
        candidates(issuer, None, printed, identities),
        printed.is_some(),
    )
}
