//! The Activity Import route against a fixture Snapshot.

mod common;

use common::{
    FakeToggl, call,
    snapshot::{Fixture, Row, uuid},
    time_app_with_snapshot, time_db,
};
use vitarchy_core::test_db::TestDb;

const IMPORT: &str = "/api/time/activity/import";

async fn stored(db: &TestDb) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM time.activities")
        .fetch_one(&db.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn an_import_stores_every_row_with_its_original_values() {
    let db = time_db("impinsert").await;
    let snap = Fixture::new();
    let mut idle = Row::new(2, 100.0, 160.5);
    idle.idle = 1;
    snap.write(&[Row::new(1, 0.0, 60.0), idle]);
    let app = time_app_with_snapshot(&db, &FakeToggl::new(), snap.file());

    let reply = call(&app, "POST", IMPORT, None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(
        (reply.json["read"].as_u64(), reply.json["inserted"].as_u64()),
        (Some(2), Some(2))
    );
    let (idle_flag, end, raw_end): (bool, String, f64) = sqlx::query_as(
        "SELECT idle, end_at::text, (raw->>'ZEND')::float8 FROM time.activities WHERE toggl_id = $1::uuid",
    )
    .bind(uuid(2))
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert!(idle_flag);
    assert_eq!(end, "2001-01-01 00:02:40.5+00");
    assert!((raw_end - 160.5).abs() < f64::EPSILON);
    db.drop().await;
}

#[tokio::test]
async fn importing_the_same_snapshot_again_changes_nothing() {
    let db = time_db("impagain").await;
    let snap = Fixture::new();
    snap.write(&[Row::new(1, 0.0, 60.0), Row::new(2, 70.0, 80.0)]);
    let app = time_app_with_snapshot(&db, &FakeToggl::new(), snap.file());
    call(&app, "POST", IMPORT, None).await;

    let reply = call(&app, "POST", IMPORT, None).await;

    assert_eq!(reply.json["inserted"], 0);
    assert_eq!(reply.json["updated"], 0);
    assert_eq!(reply.json["unchanged"], 2);
    assert_eq!(stored(&db).await, 2);
    db.drop().await;
}

#[tokio::test]
async fn an_ongoing_activity_that_grew_gets_its_new_end() {
    let db = time_db("impgrow").await;
    let snap = Fixture::new();
    snap.write(&[Row::new(1, 0.0, 60.0)]);
    let app = time_app_with_snapshot(&db, &FakeToggl::new(), snap.file());
    call(&app, "POST", IMPORT, None).await;
    snap.write(&[Row::new(1, 0.0, 600.0)]);

    let reply = call(&app, "POST", IMPORT, None).await;

    assert_eq!(
        (
            reply.json["updated"].as_u64(),
            reply.json["inserted"].as_u64()
        ),
        (Some(1), Some(0))
    );
    let end: String = sqlx::query_scalar("SELECT end_at::text FROM time.activities")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(end, "2001-01-01 00:10:00+00");
    db.drop().await;
}

#[tokio::test]
async fn activity_missing_from_a_newer_snapshot_is_kept() {
    let db = time_db("impkeep").await;
    let snap = Fixture::new();
    snap.write(&[Row::new(1, 0.0, 60.0), Row::new(2, 70.0, 80.0)]);
    let app = time_app_with_snapshot(&db, &FakeToggl::new(), snap.file());
    call(&app, "POST", IMPORT, None).await;
    snap.write(&[Row::new(2, 70.0, 80.0), Row::new(3, 90.0, 99.0)]);

    let reply = call(&app, "POST", IMPORT, None).await;

    assert_eq!(reply.json["inserted"], 1);
    assert_eq!(stored(&db).await, 3);
    db.drop().await;
}

#[tokio::test]
async fn unusable_rows_are_skipped_and_counted() {
    let db = time_db("impskip").await;
    let snap = Fixture::new();
    snap.write(&[Row::new(1, 0.0, 60.0)]);
    snap.add_row_with_bad_id();
    snap.add_backwards_row();
    let app = time_app_with_snapshot(&db, &FakeToggl::new(), snap.file());

    let reply = call(&app, "POST", IMPORT, None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(reply.json["read"], 3);
    assert_eq!(reply.json["inserted"], 1);
    assert_eq!(reply.json["skipped"], 2);
    db.drop().await;
}

#[tokio::test]
async fn without_a_snapshot_the_import_says_so_and_records_the_failure() {
    let db = time_db("impnone").await;
    let snap = Fixture::new(); // nothing written
    let app = time_app_with_snapshot(&db, &FakeToggl::new(), snap.file());

    let reply = call(&app, "POST", IMPORT, None).await;

    assert_eq!(reply.status, 404);
    assert_eq!(reply.json["code"], "no_snapshot");
    let status = call(&app, "GET", "/api/time/status", None).await;
    assert_eq!(status.json["activity"]["last_import"]["outcome"], "failed");
    assert_eq!(
        status.json["activity"]["last_import"]["error_code"],
        "no_snapshot"
    );
    db.drop().await;
}

#[tokio::test]
async fn a_file_that_is_not_a_snapshot_is_reported_as_unreadable() {
    let db = time_db("impgarbage").await;
    let snap = Fixture::new();
    snap.write_garbage();
    let app = time_app_with_snapshot(&db, &FakeToggl::new(), snap.file());

    let reply = call(&app, "POST", IMPORT, None).await;

    assert_eq!(reply.status, 422);
    assert_eq!(reply.json["code"], "snapshot_unreadable");
    db.drop().await;
}

#[tokio::test]
async fn the_status_reflects_the_imported_activity_and_the_snapshot_age() {
    let db = time_db("impstatus").await;
    let snap = Fixture::new();
    let mut other = Row::new(2, 100.0, 200.0);
    other.app = "AppTwo";
    snap.write(&[Row::new(1, 0.0, 60.0), other]);
    let app = time_app_with_snapshot(&db, &FakeToggl::new(), snap.file());
    call(&app, "POST", IMPORT, None).await;

    let status = call(&app, "GET", "/api/time/status", None).await;

    let activity = &status.json["activity"];
    assert_eq!(activity["activity_count"], 2);
    assert_eq!(activity["app_count"], 2);
    assert_eq!(activity["oldest"], "2001-01-01T00:00:00Z");
    assert_eq!(activity["newest"], "2001-01-01T00:03:20Z");
    assert!(activity["snapshot_taken_at"].is_string());
    assert_eq!(activity["last_import"]["outcome"], "success");
    assert_eq!(activity["last_import"]["counts"]["inserted"], 2);
    db.drop().await;
}
