//! Activity from several Sources: they stay apart, and the routes know only real Sources.

mod common;

use common::{
    FakeToggl,
    activity::{CLOCKIFY_IMPORT, TOGGL_IMPORT, stored_for, toggl_app},
    call,
    clockify_snapshot::{ClockifyFixture, Item},
    snapshot::{Fixture, Row, uuid},
    time_app_with_snapshots, time_db,
};
use vitarchy_time::Source;

#[tokio::test]
async fn a_toggl_row_has_no_url_or_idle_seconds() {
    let db = time_db("toggextras").await;
    let snap = Fixture::new();
    snap.write(&[Row::new(1, 0.0, 60.0)]);
    call(&toggl_app(&db, snap.file()), "POST", TOGGL_IMPORT, None).await;

    let extras: (Option<String>, Option<f64>) =
        sqlx::query_as("SELECT url, idle_secs FROM time.activities")
            .fetch_one(&db.pool)
            .await
            .unwrap();

    assert_eq!(extras, (None, None));
    db.drop().await;
}

#[tokio::test]
async fn the_same_source_id_under_two_sources_stores_two_rows() {
    let db = time_db("twosrc").await;
    let toggl = Fixture::new();
    toggl.write(&[Row::new(1, 0.0, 60.0)]);
    let clockify = ClockifyFixture::new();
    clockify.write(&[Item::new(&uuid(1), 0, 60)]);
    let app = time_app_with_snapshots(
        &db,
        &FakeToggl::new(),
        [
            (Source::Toggl, toggl.file()),
            (Source::Clockify, clockify.file()),
        ],
    );

    call(&app, "POST", TOGGL_IMPORT, None).await;
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    assert_eq!(
        (
            stored_for(&db, "toggl").await,
            stored_for(&db, "clockify").await
        ),
        (1, 1)
    );
    db.drop().await;
}

#[tokio::test]
async fn a_failed_clockify_import_shows_only_on_clockifys_status_entry() {
    let db = time_db("clknone").await;
    let toggl = Fixture::new();
    toggl.write(&[Row::new(1, 0.0, 60.0)]);
    let clockify = ClockifyFixture::new(); // nothing written
    let app = time_app_with_snapshots(
        &db,
        &FakeToggl::new(),
        [
            (Source::Toggl, toggl.file()),
            (Source::Clockify, clockify.file()),
        ],
    );
    call(&app, "POST", TOGGL_IMPORT, None).await;

    let reply = call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (404, Some("no_snapshot"))
    );
    let status = call(&app, "GET", "/api/time/status", None).await;
    let (toggl_entry, clockify_entry) = (&status.json["activity"][0], &status.json["activity"][1]);
    assert_eq!(clockify_entry["last_import"]["outcome"], "failed");
    assert_eq!(clockify_entry["last_import"]["error_code"], "no_snapshot");
    assert_eq!(toggl_entry["last_import"]["outcome"], "success");
    db.drop().await;
}

#[tokio::test]
async fn the_status_lists_both_sources_in_order_each_with_its_own_figures() {
    let db = time_db("clkstatus").await;
    let toggl = Fixture::new();
    toggl.write(&[Row::new(1, 0.0, 60.0)]);
    let clockify = ClockifyFixture::new();
    let mut other = Item::new("b", 100, 200);
    other.name = "Ghostty";
    clockify.write(&[Item::new("a", 0, 60), other]);
    let app = time_app_with_snapshots(
        &db,
        &FakeToggl::new(),
        [
            (Source::Toggl, toggl.file()),
            (Source::Clockify, clockify.file()),
        ],
    );
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;

    let status = call(&app, "GET", "/api/time/status", None).await;

    let sources: Vec<_> = status.json["activity"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["source"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(sources, ["toggl", "clockify"]);
    let (toggl_entry, clockify_entry) = (&status.json["activity"][0], &status.json["activity"][1]);
    assert_eq!(toggl_entry["activity_count"], 0);
    assert!(toggl_entry["last_import"].is_null());
    assert_eq!(
        (
            clockify_entry["activity_count"].as_i64(),
            clockify_entry["app_count"].as_i64()
        ),
        (Some(2), Some(2))
    );
    assert_eq!(clockify_entry["newest"], "2001-01-01T00:03:20Z");
    assert!(clockify_entry["snapshot_taken_at"].is_string());
    db.drop().await;
}

#[tokio::test]
async fn an_unknown_source_is_not_found() {
    let db = time_db("nosrc").await;
    let app = time_app_with_snapshots(&db, &FakeToggl::new(), []);

    let reply = call(&app, "POST", "/api/time/activity/harvest/import", None).await;

    assert_eq!(reply.status, 404);
    db.drop().await;
}

#[tokio::test]
async fn the_old_unnamed_import_route_is_gone() {
    let db = time_db("oldroute").await;
    let app = time_app_with_snapshots(&db, &FakeToggl::new(), []);

    let reply = call(&app, "POST", "/api/time/activity/import", None).await;

    assert!(
        reply.status == 404 || reply.status == 405,
        "{}",
        reply.status
    );
    db.drop().await;
}
