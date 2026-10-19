//! Hiding an Account and showing it again.

use sqlx::PgPool;

use super::{
    after::after_accounts,
    trigger::{Cause, Trigger},
};
use crate::{
    error::FinanceError,
    repo::{accounts, transfers},
};

/// Hides or shows an Account. Hiding takes it out of every Transfer at once; showing it
/// again looks for Transfers near its dates. `false` when there is no such Account.
///
/// # Errors
/// The database failed. Nothing changes then.
pub async fn set_hidden(
    pool: &PgPool,
    account_id: i64,
    hidden: bool,
    request_id: Option<&str>,
) -> Result<bool, FinanceError> {
    let mut tx = pool.begin().await?;
    if !accounts::set_hidden(&mut *tx, account_id, hidden).await? {
        return Ok(false);
    }
    let left = if hidden {
        transfers::leave_account(&mut tx, account_id).await?
    } else {
        0
    };
    tx.commit().await?;
    tracing::info!(
        account_id,
        hidden,
        transfers_dissolved = left,
        "account visibility changed"
    );
    if !hidden {
        after_accounts(
            pool,
            Cause::in_request(Trigger::Unhide, request_id),
            &[account_id],
        )
        .await;
    }
    Ok(true)
}
