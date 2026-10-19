//! Linking a Transfer by hand: the person picks the debit and the credits, and detection's
//! rules about uniqueness and unlinked pairs do not apply.

use sqlx::PgPool;

use crate::{
    error::FinanceError,
    problem::Refusal,
    repo::{
        transfer_facts::{Facts, facts},
        transfers, transfers_load,
    },
    transfers::{Invalid, Parts, Rung, validate_manual},
};

/// Links `credit_ids` to the Money-out Transaction `debit_id` as a Transfer, partial or
/// complete.
///
/// # Errors
/// A Transaction is missing, hidden, no longer found or already linked, the credits do not
/// fit the debit (see [`validate_manual`]), or the database failed. Nothing is written then.
pub async fn link(
    pool: &PgPool,
    debit_id: i64,
    credit_ids: &[i64],
    request_id: Option<&str>,
) -> Result<Parts, FinanceError> {
    let mut tx = pool.begin().await?;
    let wanted: Vec<i64> = std::iter::once(debit_id)
        .chain(credit_ids.iter().copied())
        .collect();
    let found = facts(&mut *tx, &wanted).await?;
    let (debit, credits) = pick(&found, debit_id, credit_ids)?;
    check_usable(debit, &credits)?;
    check_fit(debit, &credits)?;
    let id = transfers::get_or_create(&mut *tx, debit_id, Rung::Manual).await?;
    transfers::add_credits(&mut *tx, id, credit_ids).await?;
    let parts = transfers_load::part(&mut *tx, id).await?;
    tx.commit().await?;
    tracing::info!(
        transfer_id = id,
        credits = credit_ids.len(),
        request_id,
        "transfer linked by hand"
    );
    parts.ok_or_else(|| sqlx::Error::RowNotFound.into())
}

/// The debit and the credits among the loaded facts, in the order asked for.
fn pick<'a>(
    found: &'a [Facts],
    debit_id: i64,
    credit_ids: &[i64],
) -> Result<(&'a Facts, Vec<&'a Facts>), Refusal> {
    let find = |id: i64| {
        found
            .iter()
            .find(|f| f.id == id)
            .ok_or(Refusal::TRANSACTION_NOT_FOUND)
    };
    let credits = credit_ids
        .iter()
        .map(|id| find(*id))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((find(debit_id)?, credits))
}

/// Every Transaction must be visible, still found and in no Transfer.
fn check_usable(debit: &Facts, credits: &[&Facts]) -> Result<(), Refusal> {
    let all = || std::iter::once(debit).chain(credits.iter().copied());
    if all().any(|f| f.no_longer_found) {
        return Err(Refusal::TRANSFER_NO_LONGER_FOUND);
    }
    if all().any(|f| f.hidden) {
        return Err(Refusal::TRANSFER_HIDDEN_ACCOUNT);
    }
    if all().any(|f| f.linked) {
        return Err(Refusal::TRANSFER_ALREADY_LINKED);
    }
    Ok(())
}

fn check_fit(debit: &Facts, credits: &[&Facts]) -> Result<(), Refusal> {
    let rows: Vec<_> = credits.iter().map(|f| f.candidate()).collect();
    let all_cards = credits.iter().all(|f| f.on_card);
    validate_manual(&debit.candidate(), &rows, all_cards).map_err(refusal_for)
}

const fn refusal_for(invalid: Invalid) -> Refusal {
    match invalid {
        Invalid::NoCredits => Refusal::TRANSFER_NO_CREDITS,
        Invalid::NotMoneyOut | Invalid::NotMoneyIn => Refusal::TRANSFER_WRONG_DIRECTION,
        Invalid::SameAccount => Refusal::TRANSFER_SAME_ACCOUNT,
        Invalid::DuplicateCredit => Refusal::TRANSFER_DUPLICATE_CREDIT,
        Invalid::TooMuch => Refusal::TRANSFER_TOO_MUCH,
    }
}
