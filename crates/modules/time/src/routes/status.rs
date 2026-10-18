//! `GET /api/time/status`: what the Time page shows. Never holds the token.

use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use vitarchy_core::{ApiError, AppState, Problem};

use super::{
    activity_status::{self, SourceActivityStatus},
    run_info::RunInfo,
    timeline_status::{self, TimelineStatus},
};
use crate::{
    activity::Snapshots,
    repo::{connection, entries, runs},
};

/// Body of `GET /api/time/status`.
#[derive(Serialize, ToSchema)]
pub struct TimeStatus {
    /// Whether a Toggl token is stored.
    pub connected: bool,
    /// The synced workspace, when connected.
    pub workspace_name: Option<String>,
    /// The latest Sync attempt.
    pub last_sync: Option<RunInfo>,
    /// Time Entries stored.
    pub time_entry_count: i64,
    /// Start of the earliest stored entry.
    pub oldest_entry: Option<DateTime<Utc>>,
    /// Start of the latest stored entry.
    pub newest_entry: Option<DateTime<Utc>>,
    /// Imported Activity, Snapshot and latest Import for each Source, in a fixed order.
    pub activity: Vec<SourceActivityStatus>,
    /// The Timeline built from that Activity.
    pub timeline: TimelineStatus,
}

/// Connection, latest runs and how much has been mirrored.
#[utoipa::path(
    get,
    path = "/status",
    tag = "time",
    responses(
        (status = 200, description = "Current status", body = TimeStatus),
        (status = 500, description = "Internal error", body = Problem),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn get_status(
    State(state): State<AppState>,
    Extension(snapshots): Extension<Snapshots>,
) -> Result<Json<TimeStatus>, ApiError> {
    let pool = &state.pool;
    let connection = connection::get(pool).await?;
    let mirrored = entries::stats(pool).await?;
    Ok(Json(TimeStatus {
        connected: connection.is_some(),
        workspace_name: connection.map(|c| c.workspace_name),
        last_sync: runs::latest(pool, "sync", None).await?.map(RunInfo::from),
        time_entry_count: mirrored.count,
        oldest_entry: mirrored.oldest,
        newest_entry: mirrored.newest,
        activity: activity_status::load_all(pool, &snapshots).await?,
        timeline: timeline_status::load(pool).await?,
    }))
}
