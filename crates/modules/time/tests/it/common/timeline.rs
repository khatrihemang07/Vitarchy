//! Helpers for the Timeline tests: both Sources in one app, and the stored Segments.

use axum::Router;
use vitarchy_core::test_db::TestDb;
use vitarchy_time::{SnapshotFile, Source};

use super::{
    FakeToggl,
    activity::{CLOCKIFY_IMPORT, TOGGL_IMPORT},
    call,
    clockify_snapshot::{ClockifyFixture, Item},
    snapshot::{Fixture, Row},
    time_app_with_snapshots,
};

/// Seconds from the Unix epoch to the Core Data epoch the fixtures count from.
const CORE_DATA_EPOCH: i64 = 978_307_200;

/// The Time routes reading both Sources' Snapshots.
pub fn both_app(db: &TestDb, toggl: SnapshotFile, clockify: SnapshotFile) -> Router {
    time_app_with_snapshots(
        db,
        &FakeToggl::new(),
        [(Source::Toggl, toggl), (Source::Clockify, clockify)],
    )
}

/// A stored Segment, with times as fixture seconds.
#[derive(Debug, PartialEq, Eq)]
pub struct Stored {
    pub start: i64,
    pub end: i64,
    pub away: String,
    pub title: String,
    pub url: Option<String>,
    pub sources: Vec<String>,
}

/// Every stored Segment in time order.
pub async fn segments(db: &TestDb) -> Vec<Stored> {
    #[allow(clippy::type_complexity)]
    let rows: Vec<(i64, i64, String, String, Option<String>, Vec<String>)> = sqlx::query_as(
        "SELECT extract(epoch FROM start_at)::bigint, extract(epoch FROM end_at)::bigint,
                away, title, url, sources FROM time.timeline_segments ORDER BY start_at",
    )
    .fetch_all(&db.pool)
    .await
    .unwrap();
    rows.into_iter()
        .map(|(start, end, away, title, url, sources)| Stored {
            start: start - CORE_DATA_EPOCH,
            end: end - CORE_DATA_EPOCH,
            away,
            title,
            url,
            sources,
        })
        .collect()
}

/// Whether any two stored Segments overlap.
pub async fn has_overlap(db: &TestDb) -> bool {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM time.timeline_segments a JOIN time.timeline_segments b
           ON a.start_at < b.start_at AND b.start_at < a.end_at)",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap()
}

/// Toggl: 0-100 "Code" titled "T". Clockify: 50-7250 "Code" with an hour of no input and a page url.
pub fn write_both(toggl: &Fixture, clockify: &ClockifyFixture) {
    let mut row = Row::new(1, 0.0, 100.0);
    row.app = "Code";
    row.title = "T";
    toggl.write(&[row]);
    let mut item = Item::new("c1", 50, 7250);
    item.url = "https://example.com";
    item.idle = 3600.0;
    clockify.write(&[item]);
}

/// Imports both Sources through the routes.
pub async fn import_both(app: &Router) {
    assert_eq!(call(app, "POST", TOGGL_IMPORT, None).await.status, 200);
    assert_eq!(call(app, "POST", CLOCKIFY_IMPORT, None).await.status, 200);
}

/// Core Data seconds (what fixtures count) for an RFC 3339 moment.
pub fn core_secs(moment: &str) -> i64 {
    moment
        .parse::<chrono::DateTime<chrono::Utc>>()
        .unwrap()
        .timestamp()
        - CORE_DATA_EPOCH
}

/// Imports `toggl` and `clockify` rows into a new app on `db` and returns the app.
pub async fn import_rows(
    db: &TestDb,
    toggl_rows: &[Row],
    clockify_items: &[Item],
) -> (Router, Fixture, ClockifyFixture) {
    let (toggl, clockify) = (Fixture::new(), ClockifyFixture::new());
    toggl.write(toggl_rows);
    clockify.write(clockify_items);
    let app = both_app(db, toggl.file(), clockify.file());
    import_both(&app).await;
    (app, toggl, clockify)
}
