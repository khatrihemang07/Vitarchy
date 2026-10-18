//! Rebuilding the Account sections of a confirmed Statement that lost them (a payment
//! history confirmed before payment apps printed Transactions). Each section goes to the
//! Account its Printings already sit on, else the Account its instrument is remembered as.

use sqlx::PgConnection;

use crate::{
    accounts::instrument_key,
    error::FinanceError,
    problem::Refusal,
    reader::{ReaderResult, contract::Statement},
    repo::{instrument_accounts, printings, statement_accounts::StatementAccountRow},
};

/// The sections `result` reads, each placed on one Account; empty when the Statement has no
/// Printings to place them by.
///
/// # Errors
/// A section's Account is not clear (`statement_sections_unplaced`), or the database failed.
pub(super) async fn rebuild(
    db: &mut PgConnection,
    statement_id: i64,
    result: &ReaderResult,
) -> Result<Vec<StatementAccountRow>, FinanceError> {
    if !printings::statement_has_any(&mut *db, statement_id).await? {
        return Ok(vec![]);
    }
    let instruments = instrument_accounts::all(&mut *db).await?;
    let mut rows = Vec::new();
    for (index, section) in result.statements.iter().enumerate() {
        let source_ids = source_ids(section);
        let mut sitting =
            printings::accounts_by_source_ids(&mut *db, statement_id, &source_ids).await?;
        if sitting.is_empty()
            && let Some(id) = instrument_key(section.account.mask.as_deref())
                .and_then(|key| instruments.get(&key).copied())
        {
            sitting.push(id);
        }
        let [account_id] = sitting[..] else {
            tracing::warn!(statement_id, position = index, "section cannot be placed");
            return Err(Refusal::SECTIONS_UNPLACED.into());
        };
        rows.push(StatementAccountRow {
            position: i32::try_from(index).unwrap_or(i32::MAX),
            account_id: Some(account_id),
            proposed_issuer: section.account.issuer.clone(),
            proposed_kind: section.account.kind.map(|k| k.id().to_owned()),
            proposed_mask: section.account.mask.clone(),
        });
    }
    tracing::info!(statement_id, sections = rows.len(), "sections rebuilt");
    Ok(rows)
}

fn source_ids(section: &Statement) -> Vec<String> {
    section
        .transactions
        .iter()
        .filter_map(|t| t.source_id.clone())
        .collect()
}
