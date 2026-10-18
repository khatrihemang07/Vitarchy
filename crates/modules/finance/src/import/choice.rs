//! What the person chose for one section when confirming, and the checks on it.

use serde::Deserialize;
use utoipa::ToSchema;

use crate::{problem::Refusal, reader::contract::AccountKind};

/// A new Account to create for a section.
#[derive(Deserialize, ToSchema)]
pub struct NewAccountInput {
    /// The issuer's name.
    pub issuer: String,
    /// `savings`, `credit_card` or `loan`.
    pub kind: String,
    /// The name the person sees.
    pub name: String,
}

/// The choice for one section as the request spells it. Leave both out to accept the proposal:
/// the matched Account, or a new one made from what the reader found.
#[derive(Deserialize, ToSchema)]
pub struct SectionChoice {
    /// Assign the section to this existing Account.
    pub account_id: Option<i64>,
    /// Create this Account for the section.
    pub new_account: Option<NewAccountInput>,
}

/// A new Account as the person entered it, trimmed and accepted.
pub struct NewAccount {
    /// The issuer's name, trimmed.
    pub issuer: String,
    /// The kind of Account.
    pub kind: AccountKind,
    /// The name, trimmed.
    pub name: String,
}

/// A choice as the person entered it, trimmed and accepted.
pub enum Choice {
    /// Accept what the reader proposed.
    Proposal,
    /// Assign to an existing Account.
    Existing(i64),
    /// Create an Account.
    New(NewAccount),
}

impl TryFrom<&SectionChoice> for Choice {
    type Error = Refusal;

    fn try_from(choice: &SectionChoice) -> Result<Self, Refusal> {
        match (choice.account_id, &choice.new_account) {
            (None, None) => Ok(Self::Proposal),
            (Some(id), None) => Ok(Self::Existing(id)),
            (None, Some(new)) => {
                let issuer = new.issuer.trim();
                let name = new.name.trim();
                let kind = AccountKind::from_id(new.kind.trim());
                match kind {
                    Some(kind) if !issuer.is_empty() && !name.is_empty() => {
                        Ok(Self::New(NewAccount {
                            issuer: issuer.to_owned(),
                            kind,
                            name: name.to_owned(),
                        }))
                    }
                    _ => Err(Refusal::INVALID_NEW_ACCOUNT),
                }
            }
            (Some(_), Some(_)) => Err(Refusal::INVALID_SECTION),
        }
    }
}
