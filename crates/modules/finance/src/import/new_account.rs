//! Creating the Account a section is confirmed to when none known fits: from the person's
//! new Account, or from what the file proposed.

use sqlx::PgConnection;

use super::choice::NewAccount;
use crate::{
    accounts::{AccountIdentity, Match, last4, remembered_digits},
    error::FinanceError,
    match_account,
    problem::Refusal,
    reader::contract::{AccountKind, Kind, ProposedAccount},
    repo::accounts,
};

/// The number a new Account is created with. A Statement's is as printed. A payment app's
/// instrument is a number only when it shows digits (`XXXXXX4321`); a label such as `ICICI R`
/// says nothing about the Account's number, so it is not kept as one.
fn number_of(kind: Option<Kind>, proposal: &ProposedAccount) -> Option<&str> {
    let mask = proposal.mask.as_deref();
    match kind {
        Some(Kind::PaymentHistory) => mask.filter(|m| remembered_digits(Some(m)).is_some()),
        _ => mask,
    }
}

/// An Account is known by issuer, kind and number, so a new one whose three match exactly
/// one Account is that Account, whatever the page sent.
pub(super) async fn create_unless_known(
    db: &mut PgConnection,
    kind: Option<Kind>,
    proposal: &ProposedAccount,
    new: &NewAccount,
    known: &[AccountIdentity],
) -> Result<i64, FinanceError> {
    let number = number_of(kind, proposal);
    match match_account(Some(&new.issuer), Some(new.kind), number, known) {
        Match::One(id) => Ok(id),
        _ => Ok(
            accounts::insert(&mut *db, &new.issuer, new.kind, number, &new.name)
                .await?
                .id,
        ),
    }
}

pub(super) async fn create_from_proposal(
    db: &mut PgConnection,
    kind: Option<Kind>,
    proposal: &ProposedAccount,
) -> Result<i64, FinanceError> {
    let (Some(issuer), Some(account_kind)) = (proposal.issuer.as_deref(), proposal.kind) else {
        return Err(Refusal::NEW_ACCOUNT_REQUIRED.into());
    };
    let number = number_of(kind, proposal);
    let name = default_name(issuer, account_kind, number);
    let row = accounts::insert(&mut *db, issuer, account_kind, number, &name).await?;
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
