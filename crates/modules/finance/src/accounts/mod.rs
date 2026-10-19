//! Matching a Statement's proposed Account to the Accounts Vitarchy already knows, on
//! issuer + kind + the trailing digits both masks show.

mod instrument;
mod mask;

pub use instrument::{find_instrument, instrument_key};

pub use mask::{last4, remembered_digits, same_number, visible_digits};

use std::{collections::HashMap, hash::BuildHasher};

use crate::reader::contract::{AccountKind, Kind, ProposedAccount};

/// How an existing Account can be recognised: its own details or a remembered identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountIdentity {
    /// The Account it stands for.
    pub account_id: i64,
    /// The issuer's name.
    pub issuer: String,
    /// The kind of Account.
    pub kind: AccountKind,
    /// The number as printed (or the remembered trailing digits); `None` for an Account
    /// created without one.
    pub mask: Option<String>,
}

/// What a proposed Account matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Match {
    /// Nothing known fits (or the proposal lacks issuer or kind).
    None,
    /// Exactly one Account fits.
    One(i64),
    /// The proposal printed no number, and exactly one Account has this issuer and kind.
    OneWithoutNumber(i64),
    /// Several Accounts fit; the person must choose.
    Several(Vec<i64>),
}

impl Match {
    /// The single Account matched, however it was matched.
    #[must_use]
    pub fn single(&self) -> Option<i64> {
        match self {
            Self::One(id) | Self::OneWithoutNumber(id) => Some(*id),
            Self::None | Self::Several(_) => None,
        }
    }
}

/// Whether two issuer names are the same, ignoring case and runs of whitespace.
#[must_use]
pub fn same_issuer(a: &str, b: &str) -> bool {
    normalised(a) == normalised(b)
}

fn normalised(issuer: &str) -> String {
    issuer
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// The Accounts that fit, each once, in first-seen order: the issuer and kind when given,
/// and the printed number when there is one (an Account without a number then never fits).
fn candidates(
    issuer: Option<&str>,
    kind: Option<AccountKind>,
    printed: Option<&str>,
    identities: &[AccountIdentity],
) -> Vec<i64> {
    let mut ids: Vec<i64> = Vec::new();
    for entry in identities {
        let number_fits = printed.is_none_or(|mask| {
            entry
                .mask
                .as_deref()
                .is_some_and(|other| same_number(mask, other))
        });
        if number_fits
            && kind.is_none_or(|kind| entry.kind == kind)
            && issuer.is_none_or(|issuer| same_issuer(&entry.issuer, issuer))
            && !ids.contains(&entry.account_id)
        {
            ids.push(entry.account_id);
        }
    }
    ids
}

/// What `ids` amount to: nothing, one Account (by number, or by issuer alone), or several.
fn matched(ids: Vec<i64>, numbered: bool) -> Match {
    match (ids.as_slice(), numbered) {
        ([], _) => Match::None,
        ([one], true) => Match::One(*one),
        ([one], false) => Match::OneWithoutNumber(*one),
        _ => Match::Several(ids),
    }
}

/// Finds the Accounts a proposal belongs to, each Account once, in first-seen order. A
/// proposal with no printed number matches by issuer and kind alone.
#[must_use]
pub fn find(
    issuer: Option<&str>,
    kind: Option<AccountKind>,
    mask: Option<&str>,
    identities: &[AccountIdentity],
) -> Match {
    let (Some(issuer), Some(kind)) = (issuer, kind) else {
        return Match::None;
    };
    let printed = mask.map(str::trim).filter(|mask| !mask.is_empty());
    matched(
        candidates(Some(issuer), Some(kind), printed, identities),
        printed.is_some(),
    )
}

/// The Account a section of a file of `kind` proposes: an instrument for a payment history,
/// an issuer, kind and number for a Statement.
#[must_use]
pub fn find_for<S: BuildHasher>(
    kind: Option<Kind>,
    proposal: &ProposedAccount,
    remembered: &HashMap<String, i64, S>,
    identities: &[AccountIdentity],
) -> Match {
    if kind == Some(Kind::PaymentHistory) {
        return find_instrument(proposal, remembered, identities);
    }
    find(
        proposal.issuer.as_deref(),
        proposal.kind,
        proposal.mask.as_deref(),
        identities,
    )
}

#[cfg(test)]
mod instrument_tests;
#[cfg(test)]
mod tests;
