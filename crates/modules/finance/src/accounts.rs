//! Matching a Statement's proposed Account to the Accounts Vitarchy already knows, on
//! issuer + kind + the trailing digits both masks show.

mod mask;

pub use mask::{last4, remembered_digits, same_number};

use crate::reader::contract::AccountKind;

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
    let mut ids: Vec<i64> = Vec::new();
    for entry in identities {
        let fits = match (printed, entry.mask.as_deref()) {
            (None, _) => true,
            (Some(mask), Some(other)) => same_number(mask, other),
            (Some(_), None) => false,
        };
        if fits
            && entry.kind == kind
            && same_issuer(&entry.issuer, issuer)
            && !ids.contains(&entry.account_id)
        {
            ids.push(entry.account_id);
        }
    }
    match (ids.as_slice(), printed) {
        ([], _) => Match::None,
        ([one], Some(_)) => Match::One(*one),
        ([one], None) => Match::OneWithoutNumber(*one),
        _ => Match::Several(ids),
    }
}

#[cfg(test)]
mod tests;
