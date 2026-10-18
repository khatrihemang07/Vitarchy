//! The Sync route against a fake Toggl.

use crate::common::{
    FakeToggl, Gate, call, connect, deleted_entry, recent_entry, running_entry, stopped_entry,
    time_app, time_db,
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

async fn age_the_cursor(db: &vitarchy_core::test_db::TestDb, days: i32) {
    sqlx::query("UPDATE time.connection SET sync_cursor = now() - make_interval(days => $1)")
        .bind(days)
        .execute(&db.pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn entries_of_other_workspaces_are_not_stored() {
    let db = time_db("syncforeign").await;
    let fake = FakeToggl::new();
    let mut foreign = stopped_entry(2, "elsewhere");
    foreign["workspace_id"] = serde_json::json!(555);
    fake.with(|w| w.entries = vec![stopped_entry(1, "mine"), foreign]);
    let app = time_app(&db, &fake);
    connect(&app).await;

    let reply = call(&app, "POST", "/api/time/sync", None).await;

    assert_eq!(
        (
            reply.json["inserted"].as_u64(),
            reply.json["skipped_other_workspace"].as_u64()
        ),
        (Some(1), Some(1))
    );
    assert_eq!(entry_count(&db).await, 1);
    db.drop().await;
}

#[tokio::test]
async fn a_cursor_too_old_for_toggl_falls_back_to_the_full_window() {
    let db = time_db("syncstale").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![stopped_entry(1, "a")]);
    let app = time_app(&db, &fake);
    connect(&app).await;
    call(&app, "POST", "/api/time/sync", None).await;
    age_the_cursor(&db, 85).await;

    let reply = call(&app, "POST", "/api/time/sync", None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let queries = fake.with(|w| w.entry_queries.clone());
    assert!(
        matches!(
            queries[..],
            [EntryQuery::Range { .. }, EntryQuery::Range { .. }]
        ),
        "{queries:?}"
    );
    db.drop().await;
}

#[tokio::test]
async fn a_full_window_sync_removes_entries_toggl_no_longer_lists() {
    let db = time_db("syncreconcile").await;
    let fake = FakeToggl::new();
    fake.with(|w| {
        w.entries = vec![
            recent_entry(1, 10),
            recent_entry(2, 20),
            stopped_entry(3, "old"),
        ];
    });
    let app = time_app(&db, &fake);
    connect(&app).await;
    call(&app, "POST", "/api/time/sync", None).await;
    assert_eq!(entry_count(&db).await, 3);
    age_the_cursor(&db, 85).await;
    fake.with(|w| w.entries = vec![recent_entry(1, 10)]);

    let reply = call(&app, "POST", "/api/time/sync", None).await;

    assert_eq!(reply.json["deleted"], 1);
    let ids: Vec<i64> = sqlx::query_scalar("SELECT toggl_id FROM time.time_entries ORDER BY 1")
        .fetch_all(&db.pool)
        .await
        .unwrap();
    // 2 is gone; 3 started before the window, so this Sync cannot judge it.
    assert_eq!(ids, [1, 3]);
    db.drop().await;
}

#[tokio::test]
async fn a_stored_entry_that_is_now_running_leaves_the_mirror_until_it_stops() {
    let db = time_db("syncnowrunning").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![recent_entry(1, 5)]);
    let app = time_app(&db, &fake);
    connect(&app).await;
    call(&app, "POST", "/api/time/sync", None).await;
    age_the_cursor(&db, 85).await;
    let mut running = running_entry(1);
    running["start"] = recent_entry(1, 5)["start"].clone();
    fake.with(|w| w.entries = vec![running]);

    call(&app, "POST", "/api/time/sync", None).await;

    assert_eq!(entry_count(&db).await, 0);
    db.drop().await;
}

#[tokio::test]
async fn a_second_sync_while_one_runs_is_refused_and_the_first_still_finishes() {
    let db = time_db("synclock").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![stopped_entry(1, "a")]);
    let app = time_app(&db, &fake);
    connect(&app).await;
    let gate = Gate::default();
    fake.with(|w| w.gate = Some(gate.clone()));
    let first = tokio::spawn({
        let app = app.clone();
        async move { call(&app, "POST", "/api/time/sync", None).await }
    });
    gate.entered.notified().await;

    let second = call(&app, "POST", "/api/time/sync", None).await;

    assert_eq!(
        (second.status, second.json["code"].as_str()),
        (409, Some("sync_in_progress"))
    );
    gate.release.notify_one();
    assert_eq!(first.await.unwrap().status, 200);
    assert_eq!(entry_count(&db).await, 1);
    db.drop().await;
}

#[tokio::test]
async fn a_sync_whose_connection_was_replaced_meanwhile_writes_nothing() {
    let db = time_db("syncswapped").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![stopped_entry(1, "a")]);
    let app = time_app(&db, &fake);
    connect(&app).await;
    let gate = Gate::default();
    fake.with(|w| w.gate = Some(gate.clone()));
    let sync = tokio::spawn({
        let app = app.clone();
        async move { call(&app, "POST", "/api/time/sync", None).await }
    });
    gate.entered.notified().await;
    fake.with(|w| w.workspace_id = 8);
    connect(&app).await;

    gate.release.notify_one();
    let reply = sync.await.unwrap();

    assert_eq!(
        (reply.status, reply.json["code"].as_str()),
        (409, Some("connection_changed"))
    );
    assert_eq!(entry_count(&db).await, 0);
    let cursor: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT sync_cursor FROM time.connection")
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert!(cursor.is_none());
    db.drop().await;
}

#[tokio::test]
async fn catalog_lists_with_a_repeated_id_still_sync() {
    let db = time_db("syncdupes").await;
    let fake = FakeToggl::new();
    fake.with(|w| {
        let project = w.projects[0].clone();
        w.projects.push(project);
    });
    let app = time_app(&db, &fake);
    connect(&app).await;

    let reply = call(&app, "POST", "/api/time/sync", None).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    db.drop().await;
}
