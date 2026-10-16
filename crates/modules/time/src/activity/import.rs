//! One Import, start to finish: read the Snapshot, parse, upsert, record the run.

use std::time::Instant;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::{counts::ImportCounts, parse::parse_all, read::read_rows, snapshot_path::SnapshotFile};
use crate::{
    error::TimeError,
    repo::{activities, runs},
};

/// Runs an Import that started at `started_at`. A `runs` row is written either way,
/// and the outcome is logged with counts and duration (never a title or app name).
///
/// # Errors
/// A missing or unreadable Snapshot, or a failed database write, after the failed run was recorded.
pub async fn run_import(
    pool: &PgPool,
    file: &SnapshotFile,
    started_at: DateTime<Utc>,
) -> Result<ImportCounts, TimeError> {
    let timer = Instant::now();
    let outcome = import(pool, file).await;
    let duration_ms = u64::try_from(timer.elapsed().as_millis()).unwrap_or(u64::MAX);
    match &outcome {
        Ok(counts) => tracing::info!(
            read = counts.read,
            inserted = counts.inserted,
            updated = counts.updated,
            unchanged = counts.unchanged,
            skipped = counts.skipped,
            duration_ms,
            "activity import finished"
        ),
        Err(error) => tracing::warn!(
            error_code = error.code(),
            duration_ms,
            "activity import failed"
        ),
    }
    runs::record(pool, "import", started_at, &outcome).await;
    outcome
}

async fn import(pool: &PgPool, file: &SnapshotFile) -> Result<ImportCounts, TimeError> {
    let rows = read_rows(file).await?;
    let parsed = parse_all(&rows);
    let written = activities::upsert_all(pool, &parsed.activities).await?;
    let to_u32 = |n: u64| u32::try_from(n).unwrap_or(u32::MAX);
    let stored = to_u32(parsed.activities.len() as u64);
    let (inserted, updated) = (to_u32(written.inserted), to_u32(written.updated));
    Ok(ImportCounts {
        read: to_u32(rows.len() as u64),
        inserted,
        updated,
        unchanged: stored.saturating_sub(inserted).saturating_sub(updated),
        skipped: to_u32(parsed.skipped as u64),
    })
}
