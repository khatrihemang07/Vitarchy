//! The Transactions a Transaction could be linked to by hand.

use sqlx::PgPool;

use crate::{
    error::FinanceError,
    problem::Refusal,
    reader::contract::Direction,
    repo::transfer_facts::{Facts, facts, near},
    transfers::fits,
};

/// How far either side of the Transaction a hand-made link may reach.
pub const REACH_DAYS: i32 = 7;

/// The unlinked Transactions on other, visible Accounts within [`REACH_DAYS`], going the
/// other way, whose amounts fit (see [`fits`]); nearest first. Empty when the Transaction
/// itself is hidden or already in a Transfer.
///
/// # Errors
/// There is no such Transaction, or the database failed.
pub async fn candidates_for(
    pool: &PgPool,
    transaction_id: i64,
) -> Result<Vec<Facts>, FinanceError> {
    let anchor = facts(pool, &[transaction_id])
        .await?
        .pop()
        .ok_or(Refusal::TRANSACTION_NOT_FOUND)?;
    if anchor.hidden || anchor.linked {
        return Ok(Vec::new());
    }
    let nearby = near(pool, &anchor, REACH_DAYS).await?;
    Ok(nearby
        .into_iter()
        .filter(|other| !other.linked && fit(&anchor, other))
        .collect())
}

fn fit(anchor: &Facts, other: &Facts) -> bool {
    match anchor.direction {
        Direction::Debit => fits(anchor.amount_paise, other.amount_paise, other.on_card),
        Direction::Credit => fits(other.amount_paise, anchor.amount_paise, anchor.on_card),
    }
}
