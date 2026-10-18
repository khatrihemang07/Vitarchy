//! One Import, start to finish: read the Snapshot, parse, upsert, rebuild the Timeline over
//! what changed, record the run.

use std::time::Instant;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::{
    counts::ImportCounts, read_snapshot::read_snapshot, snapshot_path::SnapshotFile, source::Source,
};
use crate::{
    error::TimeError,
    repo::{activities, runs, timeline as timeline_repo},
    timeline,
};

/// Runs an Import of `source` that started at `started_at`. A `runs` row is written either
/// way, and the outcome is logged with the Source, counts and duration (never a title,
/// url or app name).
///
/// # Errors
/// A missing or unreadable Snapshot, or a failed database write, after the failed run was recorded.
pub async fn run_import(
    pool: &PgPool,
    source: Source,
    file: &SnapshotFile,
    started_at: DateTime<Utc>,
) -> Result<ImportCounts, TimeError> {
    let timer = Instant::now();
    let outcome = import(pool, source, file).await;
    let duration_ms = u64::try_from(timer.elapsed().as_millis()).unwrap_or(u64::MAX);
    match &outcome {
        Ok(counts) => tracing::info!(
            source = source.id(),
            read = counts.read,
            inserted = counts.inserted,
            updated = counts.updated,
            unchanged = counts.unchanged,
            skipped = counts.skipped,
            duration_ms,
            "activity import finished"
        ),
        Err(error) => tracing::warn!(
            source = source.id(),
            error_code = error.code(),
            duration_ms,
            "activity import failed"
        ),
    }
    runs::record(pool, "import", Some(source), started_at, &outcome).await;
    outcome
}

async fn import(
    pool: &PgPool,
    source: Source,
    file: &SnapshotFile,
) -> Result<ImportCounts, TimeError> {
    let rows = read_snapshot(source, file).await?;
    let activities = &rows.parsed.activities;
    // One transaction, so a failed rebuild also undoes the upsert: the next Import then finds
    // the rows changed again and retries both, and the Timeline cannot stay stale.
    let mut tx = pool.begin().await?;
    timeline_repo::lock_rebuilds(&mut tx).await?;
    let written = activities::upsert_all(&mut tx, activities).await?;
    // Stored Activity changed (where it was and where it is), so the Timeline there is stale.
    if let Some(range) = written.range {
        timeline::rebuild_within(&mut tx, range).await?;
    }
    tx.commit().await?;
    Ok(ImportCounts::tally(
        rows.read,
        activities.len(),
        &written,
        rows.parsed.skipped,
    ))
}
