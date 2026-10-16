//! One Sync, start to finish: read from Toggl, plan, write, record the run.

use std::time::Instant;

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use super::{apply::apply, counts::SyncCounts, fetch::fetch, plan::plan};
use crate::{
    error::TimeError,
    repo::{connection::Connection, runs, sync_lock},
    toggl::api::TogglApi,
};

/// Runs a Sync that started at `started_at`. Only one runs at a time. The cursor
/// moves to `started_at` only when everything succeeded. A `runs` row is written
/// either way, and the outcome is logged with counts and duration (never the
/// token or any entry text).
///
/// # Errors
/// [`TimeError::SyncInProgress`] (nothing is recorded: the running Sync records
/// itself); otherwise a failed Toggl call or database write, after the failed
/// run was recorded.
pub async fn run_sync(
    pool: &PgPool,
    api: &dyn TogglApi,
    connection: &Connection,
    started_at: DateTime<Utc>,
) -> Result<SyncCounts, TimeError> {
    let mut tx = pool.begin().await?;
    if !sync_lock::try_acquire(&mut tx).await? {
        tracing::warn!(
            workspace_id = connection.workspace_id,
            "sync refused: another is running"
        );
        return Err(TimeError::SyncInProgress);
    }
    let timer = Instant::now();
    let outcome = sync(&mut tx, api, connection, started_at).await;
    let outcome = match outcome {
        Ok(counts) => tx.commit().await.map(|()| counts).map_err(TimeError::from),
        Err(error) => Err(error),
    };
    let duration_ms = u64::try_from(timer.elapsed().as_millis()).unwrap_or(u64::MAX);
    match &outcome {
        Ok(counts) => tracing::info!(
            workspace_id = connection.workspace_id,
            inserted = counts.inserted,
            updated = counts.updated,
            deleted = counts.deleted,
            skipped_running = counts.skipped_running,
            skipped_other_workspace = counts.skipped_other_workspace,
            duration_ms,
            "sync finished"
        ),
        Err(error) => tracing::warn!(
            workspace_id = connection.workspace_id,
            error_code = error.code(),
            duration_ms,
            "sync failed"
        ),
    }
    runs::record(pool, "sync", started_at, &outcome).await;
    outcome
}

async fn sync(
    tx: &mut PgConnection,
    api: &dyn TogglApi,
    connection: &Connection,
    started_at: DateTime<Utc>,
) -> Result<SyncCounts, TimeError> {
    let mut fetched = fetch(api, connection, started_at).await?;
    let planned = plan(
        std::mem::take(&mut fetched.entries),
        connection.workspace_id,
        &fetched.query,
    );
    apply(tx, connection, &fetched, &planned, started_at).await
}
