//! The Activity part of the status response.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;

use super::run_info::RunInfo;
use crate::{
    activity::SnapshotFile,
    repo::{activities, runs},
};

/// Activity figures for the status page.
#[derive(Serialize, ToSchema)]
pub struct ActivityStatus {
    /// Activity rows stored.
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

/// Stored Activity, the Snapshot's age and the latest Import.
pub async fn load(pool: &PgPool, snapshot: &SnapshotFile) -> Result<ActivityStatus, sqlx::Error> {
    let stored = activities::stats(pool).await?;
    Ok(ActivityStatus {
        activity_count: stored.count,
        oldest: stored.oldest,
        newest: stored.newest,
        app_count: stored.app_count,
        snapshot_taken_at: snapshot.taken_at().await,
        last_import: runs::latest(pool, "import").await?.map(RunInfo::from),
    })
}
