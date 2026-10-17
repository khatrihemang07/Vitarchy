//! One Import, start to finish: read the Snapshot, parse, upsert, record the run.

use std::time::Instant;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::{
    counts::ImportCounts, read_snapshot::read_snapshot, snapshot_path::SnapshotFile, source::Source,
};
use crate::{
    error::TimeError,
    repo::{activities, runs},
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
    let written = activities::upsert_all(pool, activities).await?;
    Ok(ImportCounts::tally(
        rows.read,
        activities.len(),
        &written,
        rows.parsed.skipped,
    ))
}
