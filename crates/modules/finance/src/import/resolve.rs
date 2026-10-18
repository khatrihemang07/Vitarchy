//! Deciding which Account a section is written to when the person confirms.

use std::collections::HashMap;

use sqlx::PgConnection;

use super::{
    choice::Choice,
    new_account::{create_from_proposal, create_unless_known},
};
use crate::{
    accounts::{AccountIdentity, Match, find_for, instrument_key, remembered_digits},
    error::FinanceError,
    problem::Refusal,
    reader::contract::{Kind, ProposedAccount},
    repo::{account_identifiers, accounts, instrument_accounts},
};

/// Everything that tells one Account from another, as the database holds it now.
pub struct Known {
    /// Every way an Account is known.
    pub accounts: Vec<AccountIdentity>,
    /// The Account each remembered instrument is, by [`instrument_key`].
    pub instruments: HashMap<String, i64>,
}

impl Known {
    /// Loads what is known now.
    ///
    /// # Errors
    /// The database failed.
    pub async fn load(db: &mut PgConnection) -> Result<Self, sqlx::Error> {
        Ok(Self {
            accounts: accounts::identities(&mut *db).await?,
            instruments: instrument_accounts::all(&mut *db).await?,
        })
    }
}

/// The Account id for a section of a file of `kind`, creating the Account or remembering an
/// identifier or instrument as needed.
///
/// # Errors
/// The choice does not fit the proposal, names a missing Account, or the database failed.
pub async fn account_for(
    db: &mut PgConnection,
    kind: Option<Kind>,
    proposal: &ProposedAccount,
    choice: &Choice,
    known: &Known,
) -> Result<i64, FinanceError> {
    let found = find_for(kind, proposal, &known.instruments, &known.accounts);
    let id = match choice {
        Choice::Existing(id) => {
            accounts::get(&mut *db, *id)
                .await?
                .ok_or(Refusal::ACCOUNT_NOT_FOUND)?;
            remember_if_new(db, proposal, &found, *id).await?;
            *id
        }
        Choice::New(new) => create_unless_known(db, kind, proposal, new, &known.accounts).await?,
        Choice::Proposal => match found {
            Match::One(id) | Match::OneWithoutNumber(id) => id,
            Match::Several(_) => return Err(Refusal::AMBIGUOUS_ACCOUNT.into()),
            Match::None => create_from_proposal(db, kind, proposal).await?,
        },
    };
    if kind == Some(Kind::PaymentHistory)
        && let Some(instrument) = instrument_key(proposal.mask.as_deref())
    {
        instrument_accounts::remember(&mut *db, &instrument, id).await?;
        tracing::debug!(account_id = id, "instrument remembered");
    }
    Ok(id)
}

/// Reassigning a section to an Account the proposal did not match teaches Vitarchy that this
/// issuer, kind and trailing digits mean that Account. An instrument has no kind, so only
/// the instrument itself is remembered (by the caller).
async fn remember_if_new(
    db: &mut PgConnection,
    proposal: &ProposedAccount,
    found: &Match,
    account_id: i64,
) -> Result<(), sqlx::Error> {
    let already = match found {
        Match::One(id) | Match::OneWithoutNumber(id) => *id == account_id,
        Match::Several(ids) => ids.contains(&account_id),
        Match::None => false,
    };
    if let (false, Some(issuer), Some(kind), Some(digits)) = (
        already,
        proposal.issuer.as_deref(),
        proposal.kind,
        remembered_digits(proposal.mask.as_deref()),
    ) {
        account_identifiers::remember(&mut *db, account_id, issuer, kind, &digits).await?;
    }
    Ok(())
}
