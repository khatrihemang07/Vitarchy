//! One detection run: read what is near the change, find Transfers, store them.

use std::time::Instant;

use chrono::NaiveDate;
use sqlx::{PgConnection, PgPool};

use super::{trigger::Cause, window::around};
use crate::{
    error::FinanceError,
    repo::{confirm_lock, detection_input, transfers, transfers_load},
    transfers::{Detection, Input, detect},
};

/// Runs detection over the Transactions near `affected` (every Transaction when `None`) and
/// stores what it finds, under the confirm lock. Logs the trigger and counts only.
///
/// # Errors
/// A confirm is running, or the database failed. Nothing is written then.
pub async fn run(
    pool: &PgPool,
    cause: Cause<'_>,
    affected: Option<(NaiveDate, NaiveDate)>,
) -> Result<Detection, FinanceError> {
    let mut tx = pool.begin().await?;
    if !confirm_lock::try_acquire(&mut tx).await? {
        return Err(FinanceError::ConfirmInProgress);
    }
    let found = run_in(&mut tx, cause, affected).await?;
    tx.commit().await?;
    Ok(found)
}

/// [`run`] inside the caller's transaction, which already holds the confirm lock.
///
/// # Errors
/// The database failed.
pub async fn run_in(
    db: &mut PgConnection,
    cause: Cause<'_>,
    affected: Option<(NaiveDate, NaiveDate)>,
) -> Result<Detection, FinanceError> {
    let timer = Instant::now();
    let window = affected.map(|(from, to)| around(from, to));
    let accounts = detection_input::accounts(&mut *db).await?;
    let candidates = detection_input::candidates(&mut *db, window.map(|w| w.load)).await?;
    let stored = transfers_load::all(&mut *db).await?;
    let rejected = transfers_load::rejected_pairs(&mut *db).await?;
    let found = detect(&Input {
        accounts: &accounts,
        candidates: &candidates,
        transfers: &stored,
        rejected: &rejected,
        focus: window.map(|w| w.focus),
    });
    for link in &found.links {
        let id = transfers::get_or_create(&mut *db, link.debit_id, link.rung).await?;
        transfers::add_credits(&mut *db, id, &link.credit_ids).await?;
    }
    tracing::info!(
        trigger = cause.trigger.as_str(),
        request_id = cause.request_id,
        candidates = candidates.len(),
        reference = found.found.reference,
        account_number = found.found.account_number,
        bill_payment = found.found.bill_payment,
        unique_amount = found.found.unique_amount,
        ambiguous = found.ambiguous,
        duration_ms = u64::try_from(timer.elapsed().as_millis()).unwrap_or(u64::MAX),
        "transfers detected"
    );
    Ok(found)
}
