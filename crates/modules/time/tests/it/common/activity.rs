//! Shared helpers for the Activity Import tests: one app per Source, and row counts.

use axum::Router;
use vitarchy_core::test_db::TestDb;
use vitarchy_time::{SnapshotFile, Source};

use super::{FakeToggl, time_app_with_snapshots};

/// The Toggl Import route.
pub const TOGGL_IMPORT: &str = "/api/time/activity/toggl/import";
/// The Clockify Import route.
pub const CLOCKIFY_IMPORT: &str = "/api/time/activity/clockify/import";

/// The Time routes reading Toggl's Snapshot from `file`.
pub fn toggl_app(db: &TestDb, file: SnapshotFile) -> Router {
    time_app_with_snapshots(db, &FakeToggl::new(), [(Source::Toggl, file)])
}

/// The Time routes reading Clockify's Snapshot from `file`.
pub fn clockify_app(db: &TestDb, file: SnapshotFile) -> Router {
    time_app_with_snapshots(db, &FakeToggl::new(), [(Source::Clockify, file)])
}

/// Activity rows stored for `source`.
pub async fn stored_for(db: &TestDb, source: &str) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM time.activities WHERE source = $1")
        .bind(source)
        .fetch_one(&db.pool)
        .await
        .unwrap()
}
