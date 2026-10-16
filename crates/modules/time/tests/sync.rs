//! The Sync route against a fake Toggl.

mod common;

use common::{
    FakeToggl, call, connect, deleted_entry, running_entry, stopped_entry, time_app, time_db,
};
use vitarchy_time::toggl::{error::TogglError, types::EntryQuery};

async fn entry_count(db: &vitarchy_core::test_db::TestDb) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM time.time_entries")
        .fetch_one(&db.pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn syncing_without_a_connection_is_a_conflict() {
    let db = time_db("syncnone").await;
    let app = time_app(&db, &FakeToggl::new());

    let reply = call(&app, "POST", "/api/time/sync", None).await;

    assert_eq!(reply.status, 409);
    db.drop().await;
}

#[tokio::test]
async fn the_first_sync_asks_for_a_date_window_and_stores_everything() {
    let db = time_db("syncfirst").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![stopped_entry(1, "a"), stopped_entry(2, "b")]);
    let app = time_app(&db, &fake);
    connect(&app).await;

    let reply = call(&app, "POST", "/api/time/sync", None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(reply.json["inserted"], 2);
    assert_eq!(
        (
            reply.json["projects"].as_u64(),
            reply.json["clients"].as_u64(),
            reply.json["tags"].as_u64()
        ),
        (Some(1), Some(1), Some(1))
    );
    assert!(matches!(
        fake.with(|w| w.entry_queries.clone())[..],
        [EntryQuery::Range { .. }]
    ));
    assert_eq!(entry_count(&db).await, 2);
    db.drop().await;
}

#[tokio::test]
async fn the_second_sync_asks_for_changes_since_and_repeating_it_changes_nothing() {
    let db = time_db("syncsecond").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![stopped_entry(1, "a")]);
    let app = time_app(&db, &fake);
    connect(&app).await;
    call(&app, "POST", "/api/time/sync", None).await;

    let reply = call(&app, "POST", "/api/time/sync", None).await;

    assert_eq!(reply.json["inserted"], 0);
    assert_eq!(reply.json["updated"], 1);
    let queries = fake.with(|w| w.entry_queries.clone());
    assert!(matches!(queries[1], EntryQuery::Since(_)), "{queries:?}");
    assert_eq!(entry_count(&db).await, 1);
    db.drop().await;
}

#[tokio::test]
async fn a_running_entry_is_skipped_then_stored_once_it_has_stopped() {
    let db = time_db("syncrunning").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![running_entry(3)]);
    let app = time_app(&db, &fake);
    connect(&app).await;

    let first = call(&app, "POST", "/api/time/sync", None).await;
    assert_eq!(
        (
            first.json["skipped_running"].as_u64(),
            first.json["inserted"].as_u64()
        ),
        (Some(1), Some(0))
    );
    assert_eq!(entry_count(&db).await, 0);

    fake.with(|w| w.entries = vec![stopped_entry(3, "now stopped")]);
    let second = call(&app, "POST", "/api/time/sync", None).await;
    assert_eq!(second.json["inserted"], 1);
    assert_eq!(entry_count(&db).await, 1);
    db.drop().await;
}

#[tokio::test]
async fn an_entry_deleted_in_toggl_is_removed() {
    let db = time_db("syncdelete").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![stopped_entry(1, "a"), stopped_entry(2, "b")]);
    let app = time_app(&db, &fake);
    connect(&app).await;
    call(&app, "POST", "/api/time/sync", None).await;

    fake.with(|w| w.entries = vec![deleted_entry(2)]);
    let reply = call(&app, "POST", "/api/time/sync", None).await;

    assert_eq!(reply.json["deleted"], 1);
    assert_eq!(entry_count(&db).await, 1);
    db.drop().await;
}

#[tokio::test]
async fn a_rate_limit_is_a_problem_with_a_reset_time_and_a_failed_run_is_recorded() {
    let db = time_db("syncrate").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![stopped_entry(1, "a")]);
    let app = time_app(&db, &fake);
    connect(&app).await;
    fake.with(|w| {
        w.fail_with = Some(TogglError::RateLimited {
            resets_in_secs: Some(1800),
        });
    });

    let reply = call(&app, "POST", "/api/time/sync", None).await;

    assert_eq!(reply.status, 429);
    assert_eq!(reply.content_type, "application/problem+json");
    assert_eq!(reply.json["code"], "rate_limited");
    assert_eq!(reply.json["resets_in_secs"], 1800);
    let status = call(&app, "GET", "/api/time/status", None).await;
    assert_eq!(status.json["last_sync"]["outcome"], "failed");
    assert_eq!(status.json["last_sync"]["error_code"], "rate_limited");
    assert_eq!(entry_count(&db).await, 0);
    db.drop().await;
}

#[tokio::test]
async fn a_failed_sync_does_not_move_the_cursor_so_the_next_one_reads_the_full_window() {
    let db = time_db("synccursor").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![stopped_entry(1, "a")]);
    let app = time_app(&db, &fake);
    connect(&app).await;
    fake.with(|w| w.fail_with = Some(TogglError::Unreachable));
    assert_eq!(call(&app, "POST", "/api/time/sync", None).await.status, 502);
    fake.with(|w| w.fail_with = None);

    call(&app, "POST", "/api/time/sync", None).await;

    let queries = fake.with(|w| w.entry_queries.clone());
    assert!(
        matches!(queries[..], [EntryQuery::Range { .. }]),
        "{queries:?}"
    );
    db.drop().await;
}

#[tokio::test]
async fn a_token_toggl_no_longer_accepts_fails_the_sync_with_invalid_token() {
    let db = time_db("syncrevoked").await;
    let fake = FakeToggl::new();
    let app = time_app(&db, &fake);
    connect(&app).await;
    fake.with(|w| w.valid_token = "rotated".to_owned());

    let reply = call(&app, "POST", "/api/time/sync", None).await;

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (401, Some("invalid_token"))
    );
    db.drop().await;
}
