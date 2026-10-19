//! The one-time run at server start that links Transfers among Transactions stored before
//! Transfers existed (ADR 0017).

use sqlx::PgPool;

use super::{
    run::run_in,
    trigger::{Cause, Trigger},
};
use crate::{
    error::FinanceError,
    repo::{confirm_lock, markers},
};

/// The marker that says the backfill has run.
pub const BACKFILL_MARKER: &str = "transfer_backfill_done";

/// Detects over everything stored and records the marker, once. Returns whether it ran now:
/// `false` when the marker was already there or a confirm held the lock (it will run at the
/// next start). The marker is written in the same transaction as the links, so a failed run
/// leaves neither.
///
/// # Errors
/// The database failed.
pub async fn backfill_once(pool: &PgPool) -> Result<bool, FinanceError> {
    let mut tx = pool.begin().await?;
    if markers::is_done(&mut *tx, BACKFILL_MARKER).await? {
        return Ok(false);
    }
    if !confirm_lock::try_acquire(&mut tx).await? {
        return Ok(false);
    }
    run_in(&mut tx, Cause::outside_request(Trigger::Backfill), None).await?;
    markers::mark_done(&mut *tx, BACKFILL_MARKER).await?;
    tx.commit().await?;
    Ok(true)
}

/// [`backfill_once`] for server start: a failure is logged and retried at the next start.
pub async fn backfill_on_start(pool: &PgPool) {
    match backfill_once(pool).await {
        Ok(true) => tracing::info!("transfer backfill done"),
        Ok(false) => tracing::debug!("transfer backfill skipped"),
        Err(error) => tracing::warn!(code = error.code(), "transfer backfill failed"),
    }
}
