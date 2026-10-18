//! The Timeline rebuilt once when the Away rules change, and not again after.

mod common;

use common::{
    activity::{CLOCKIFY_IMPORT, clockify_app},
    call,
    clockify_snapshot::{ClockifyFixture, Item},
    time_db,
    timeline::segments,
};
use vitarchy_core::test_db::TestDb;

const STATUS: &str = "/api/time/status";
const SUMMARY: &str = "/api/time/timeline?kind=day&from=2026-10-17&to=2026-10-17&tz=UTC";

async fn stored_version(db: &TestDb) -> Option<String> {
    sqlx::query_scalar("SELECT rules_fingerprint FROM time.timeline_meta")
        .fetch_optional(&db.pool)
        .await
        .unwrap()
}

async fn import_ninety_minutes_away(db: &TestDb) -> axum::Router {
    let clockify = ClockifyFixture::new();
    let mut item = Item::new("c1", 0, 5400);
    item.idle = 5400.0;
    clockify.write(&[item]);
    let app = clockify_app(db, clockify.file());
    call(&app, "POST", CLOCKIFY_IMPORT, None).await;
    app
}

async fn drop_segments(db: &TestDb) {
    sqlx::query("DELETE FROM time.timeline_segments")
        .execute(&db.pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn a_timeline_built_under_other_rules_is_rebuilt_once_on_the_next_read() {
    let db = time_db("tlver_status").await;
    let app = import_ninety_minutes_away(&db).await;
    drop_segments(&db).await;
    assert_eq!(stored_version(&db).await, None);

    let first = call(&app, "GET", STATUS, None).await;
    assert_eq!(first.status, 200, "{}", first.text);

    assert_eq!(segments(&db).await.len(), 1);
    assert!(stored_version(&db).await.is_some());
    drop_segments(&db).await;
    call(&app, "GET", STATUS, None).await;
    call(&app, "GET", SUMMARY, None).await;
    assert!(
        segments(&db).await.is_empty(),
        "the second read must not rebuild"
    );
    db.drop().await;
}

#[tokio::test]
async fn a_different_stored_version_triggers_a_rebuild_from_the_timeline_summary() {
    let db = time_db("tlver_summary").await;
    let app = import_ninety_minutes_away(&db).await;
    call(&app, "GET", STATUS, None).await;
    sqlx::query("UPDATE time.timeline_meta SET rules_fingerprint = rules_fingerprint || '-old'")
        .execute(&db.pool)
        .await
        .unwrap();
    drop_segments(&db).await;

    let reply = call(&app, "GET", SUMMARY, None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(segments(&db).await.len(), 1);
    db.drop().await;
}

#[tokio::test]
async fn a_failed_rebuild_does_not_break_the_status_or_the_timeline() {
    let db = time_db("tlver_fail").await;
    let app = import_ninety_minutes_away(&db).await;
    let before = segments(&db).await;
    sqlx::query("DELETE FROM time.timeline_meta")
        .execute(&db.pool)
        .await
        .unwrap();
    for statement in [
        "CREATE FUNCTION time.refuse_segments() RETURNS trigger LANGUAGE plpgsql
           AS $$ BEGIN RAISE EXCEPTION 'segments refused'; END $$",
        "CREATE TRIGGER refuse BEFORE INSERT ON time.timeline_segments
           FOR EACH ROW EXECUTE FUNCTION time.refuse_segments()",
    ] {
        sqlx::query(statement).execute(&db.pool).await.unwrap();
    }

    let status = call(&app, "GET", STATUS, None).await;
    let summary = call(&app, "GET", SUMMARY, None).await;

    assert_eq!(status.status, 200, "{}", status.text);
    assert_eq!(summary.status, 200, "{}", summary.text);
    assert_eq!(
        segments(&db).await,
        before,
        "the stored Timeline is untouched"
    );
    assert_eq!(
        stored_version(&db).await,
        None,
        "so the next read tries again"
    );
    db.drop().await;
}
