//! One Day recorded by both Sources, for the Timeline summary tests.

use axum::Router;
use vitarchy_core::test_db::TestDb;

use super::{
    clockify_snapshot::Item,
    snapshot::Row,
    timeline::{core_secs, import_rows},
};

/// Fixture seconds as the float a Toggl row holds (they fit f64 exactly).
fn secs_f64(moment: &str) -> f64 {
    f64::from(i32::try_from(core_secs(moment)).unwrap())
}

pub fn clockify(id: &str, from: &str, to: &str, app: &'static str, idle: f64) -> Item {
    let mut item = Item::new(id, core_secs(from), core_secs(to));
    item.name = app;
    item.idle = idle;
    item
}

fn toggl(id: u8, from: &str, to: &str) -> Row {
    let mut row = Row::new(id, secs_f64(from), secs_f64(to));
    row.app = "Code";
    row.title = "T";
    row
}

/// 09:00-09:30 Code (both Sources), 09:30-09:40 Mail, 09:40-10:40 Away (an hour without input), 10:40-10:45 Toggl only.
pub async fn two_source_day(db: &TestDb) -> Router {
    let items = [
        clockify(
            "c1",
            "2026-10-17T09:00:00Z",
            "2026-10-17T09:30:00Z",
            "Code",
            0.0,
        ),
        clockify(
            "c2",
            "2026-10-17T09:30:00Z",
            "2026-10-17T09:40:00Z",
            "Mail",
            0.0,
        ),
        clockify(
            "c3",
            "2026-10-17T09:40:00Z",
            "2026-10-17T10:40:00Z",
            "Lock",
            3600.0,
        ),
    ];
    let rows = [
        toggl(1, "2026-10-17T09:00:00Z", "2026-10-17T09:30:00Z"),
        toggl(2, "2026-10-17T10:40:00Z", "2026-10-17T10:45:00Z"),
    ];
    import_rows(db, &rows, &items).await.0
}
