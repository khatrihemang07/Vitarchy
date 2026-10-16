//! One Sync, start to finish: read from Toggl, plan, write, record the run.

use std::time::Instant;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::{apply::apply, counts::SyncCounts, fetch::fetch, plan::plan};
use crate::{
    error::TimeError,
    repo::{connection::Connection, runs},
    toggl::api::TogglApi,
};

/// Runs a Sync that started at `started_at`. The cursor moves to `started_at`
/// only when everything succeeded. A `runs` row is written either way, and the
/// outcome is logged with counts and duration (never the token or any entry text).
///
/// # Errors
/// A failed Toggl call or database write, after the failed run was recorded.
pub async fn run_sync(
    pool: &PgPool,
    api: &dyn TogglApi,
    connection: &Connection,
    started_at: DateTime<Utc>,
) -> Result<SyncCounts, TimeError> {
    let timer = Instant::now();
    let outcome = sync(pool, api, connection, started_at).await;
    let duration_ms = u64::try_from(timer.elapsed().as_millis()).unwrap_or(u64::MAX);
    match &outcome {
        Ok(counts) => tracing::info!(
            workspace_id = connection.workspace_id,
            inserted = counts.inserted,
            updated = counts.updated,
            deleted = counts.deleted,
            skipped_running = counts.skipped_running,
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
    pool: &PgPool,
    api: &dyn TogglApi,
    connection: &Connection,
    started_at: DateTime<Utc>,
) -> Result<SyncCounts, TimeError> {
    let mut fetched = fetch(api, connection, started_at).await?;
    let planned = plan(
        std::mem::take(&mut fetched.entries),
        connection.workspace_id,
    );
    Ok(apply(
        pool,
        connection.workspace_id,
        &fetched,
        &planned,
        started_at,
    )
    .await?)
}
