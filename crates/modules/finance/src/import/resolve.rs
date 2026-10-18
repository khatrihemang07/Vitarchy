//! Deciding which Account a section is written to when the person confirms.

use sqlx::PgConnection;

use super::choice::{Choice, NewAccount};
use crate::{
    accounts::{AccountIdentity, Match, last4, remembered_digits},
    error::FinanceError,
    match_account,
    problem::Refusal,
    reader::contract::{AccountKind, ProposedAccount},
    repo::{account_identifiers, accounts},
};

/// The Account id for a section, creating the Account or remembering an identifier as needed.
///
/// # Errors
/// The choice does not fit the proposal, names a missing Account, or the database failed.
pub async fn account_for(
    db: &mut PgConnection,
    proposal: &ProposedAccount,
    choice: &Choice,
    known: &[AccountIdentity],
) -> Result<i64, FinanceError> {
    let found = match_account(
        proposal.issuer.as_deref(),
        proposal.kind,
        proposal.mask.as_deref(),
        known,
    );
    match choice {
        Choice::Existing(id) => {
            accounts::get(&mut *db, *id)
                .await?
                .ok_or(Refusal::ACCOUNT_NOT_FOUND)?;
            remember_if_new(db, proposal, &found, *id).await?;
            Ok(*id)
        }
        Choice::New(new) => create_unless_known(db, proposal, new, known).await,
        Choice::Proposal => match found {
            Match::One(id) | Match::OneWithoutNumber(id) => Ok(id),
            Match::Several(_) => Err(Refusal::AMBIGUOUS_ACCOUNT.into()),
            Match::None => create_from_proposal(db, proposal).await,
        },
    }
}

/// An Account is known by issuer, kind and number, so a new one whose three match exactly
/// one Account is that Account, whatever the page sent.
async fn create_unless_known(
    db: &mut PgConnection,
    proposal: &ProposedAccount,
    new: &NewAccount,
    known: &[AccountIdentity],
) -> Result<i64, FinanceError> {
    let same = match_account(
        Some(&new.issuer),
        Some(new.kind),
        proposal.mask.as_deref(),
        known,
    );
    match same {
        Match::One(id) => Ok(id),
        _ => create(db, proposal, new).await,
    }
}

async fn create(
    db: &mut PgConnection,
    proposal: &ProposedAccount,
    new: &NewAccount,
) -> Result<i64, FinanceError> {
    let row = accounts::insert(
        &mut *db,
        &new.issuer,
        new.kind,
        proposal.mask.as_deref(),
        &new.name,
    )
    .await?;
    Ok(row.id)
}

async fn create_from_proposal(
    db: &mut PgConnection,
    proposal: &ProposedAccount,
) -> Result<i64, FinanceError> {
    let (Some(issuer), Some(kind)) = (proposal.issuer.as_deref(), proposal.kind) else {
        return Err(Refusal::NEW_ACCOUNT_REQUIRED.into());
    };
    let name = default_name(issuer, kind, proposal.mask.as_deref());
    let row = accounts::insert(&mut *db, issuer, kind, proposal.mask.as_deref(), &name).await?;
    Ok(row.id)
}

/// `ICICI Bank credit card 2003`: the issuer, the kind in words and the last 4 when known.
fn default_name(issuer: &str, kind: AccountKind, mask: Option<&str>) -> String {
    let kind = kind.id().replace('_', " ");
    match last4(mask) {
        Some(digits) => format!("{issuer} {kind} {digits}"),
        None => format!("{issuer} {kind}"),
    }
}

/// Reassigning a section to an Account the proposal did not match teaches Vitarchy that this
/// issuer, kind and trailing digits mean that Account.
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
