//! Writing a Statement's Account proposals: what the reader found, and the Account it matched.

use sqlx::PgConnection;

use crate::{
    accounts::{Match, find_for},
    reader::contract::{Kind, Statement},
    repo::{
        accounts, instrument_accounts,
        statement_accounts::{self, StatementAccountRow},
    },
};

/// Stores one slot per section, pointing at the Account each matched exactly. A payment
/// app's sections are matched live (a later confirm may teach Vitarchy an instrument), so
/// their slots stay empty until the person confirms.
///
/// # Errors
/// The database failed.
pub async fn propose(
    db: &mut PgConnection,
    statement_id: i64,
    kind: Option<Kind>,
    sections: &[Statement],
) -> Result<(), sqlx::Error> {
    let known = accounts::identities(&mut *db).await?;
    let instruments = instrument_accounts::all(&mut *db).await?;
    for (index, section) in sections.iter().enumerate() {
        let proposal = &section.account;
        let found = find_for(kind, proposal, &instruments, &known);
        let row = StatementAccountRow {
            position: i32::try_from(index).unwrap_or(i32::MAX),
            // A match by issuer and kind alone is left for the preview to flag, so it is
            // recomputed live and the person sees that no number was printed.
            account_id: match found {
                Match::One(id) if kind != Some(Kind::PaymentHistory) => Some(id),
                _ => None,
            },
            proposed_issuer: proposal.issuer.clone(),
            proposed_kind: proposal.kind.map(|k| k.id().to_owned()),
            proposed_mask: proposal.mask.clone(),
        };
        statement_accounts::insert(&mut *db, statement_id, &row).await?;
    }
    Ok(())
}
