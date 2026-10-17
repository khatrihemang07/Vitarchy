//! The Activity part of the status response.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;

use super::run_info::RunInfo;
use crate::{
    activity::{SnapshotFile, Snapshots, Source},
    repo::{
        activities::{self, Stats},
        runs::{self, RunRow},
    },
};

/// One Source's Activity figures for the status page.
#[derive(Serialize, ToSchema)]
pub struct ActivityStatus {
    /// Activity rows stored for this Source.
    pub activity_count: i64,
    /// Start of the earliest stored Activity.
    pub oldest: Option<DateTime<Utc>>,
    /// End of the latest stored Activity.
    pub newest: Option<DateTime<Utc>>,
    /// Distinct apps seen.
    pub app_count: i64,
    /// When the Snapshot was taken; `null` when there is no Snapshot.
    pub snapshot_taken_at: Option<DateTime<Utc>>,
    /// The latest Import attempt.
    pub last_import: Option<RunInfo>,
}

/// [`ActivityStatus`] naming the Source it describes.
#[derive(Serialize, ToSchema)]
pub struct SourceActivityStatus {
    /// Which Source.
    pub source: Source,
    /// Its figures.
    #[serde(flatten)]
    pub status: ActivityStatus,
}

/// Every Source's status, in descriptor order.
pub async fn load_all(
    pool: &PgPool,
    snapshots: &Snapshots,
) -> Result<Vec<SourceActivityStatus>, sqlx::Error> {
    let mut statuses = Vec::new();
    for source in Source::all() {
        statuses.push(SourceActivityStatus {
            source,
            status: load(pool, source, snapshots.get(source)).await?,
        });
    }
    Ok(statuses)
}

/// One Source's stored Activity, its Snapshot's age and its latest Import.
async fn load(
    pool: &PgPool,
    source: Source,
    snapshot: &SnapshotFile,
) -> Result<ActivityStatus, sqlx::Error> {
    let stored = activities::stats(pool, source).await?;
    let last_import = runs::latest(pool, "import", Some(source)).await?;
    Ok(assemble(&stored, snapshot.taken_at().await, last_import))
}

fn assemble(
    stored: &Stats,
    snapshot_taken_at: Option<DateTime<Utc>>,
    last_import: Option<RunRow>,
) -> ActivityStatus {
    ActivityStatus {
        activity_count: stored.count,
        oldest: stored.oldest,
        newest: stored.newest,
        app_count: stored.app_count,
        snapshot_taken_at,
        last_import: last_import.map(RunInfo::from),
    }
}
