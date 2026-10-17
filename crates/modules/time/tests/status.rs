//! The status route.

mod common;

use common::{FakeToggl, TOKEN, call, connect, stopped_entry, time_app, time_db};

#[tokio::test]
async fn before_connecting_the_status_is_empty() {
    let db = time_db("statusempty").await;
    let app = time_app(&db, &FakeToggl::new());

    let reply = call(&app, "GET", "/api/time/status", None).await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.json["connected"], false);
    assert!(reply.json["last_sync"].is_null());
    assert_eq!(reply.json["time_entry_count"], 0);
    assert!(reply.json["oldest_entry"].is_null());
    for activity in reply.json["activity"].as_array().unwrap() {
        assert_eq!(activity["activity_count"], 0);
        assert_eq!(activity["app_count"], 0);
        assert!(activity["oldest"].is_null() && activity["snapshot_taken_at"].is_null());
        assert!(activity["last_import"].is_null());
    }
    db.drop().await;
}

#[tokio::test]
async fn the_status_never_contains_the_token() {
    let db = time_db("statustoken").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![stopped_entry(1, "work")]);
    let app = time_app(&db, &fake);
    connect(&app).await;
    call(&app, "POST", "/api/time/sync", None).await;

    let reply = call(&app, "GET", "/api/time/status", None).await;

    assert!(!reply.text.contains(TOKEN), "{}", reply.text);
    assert!(!reply.text.contains("token"), "{}", reply.text);
    db.drop().await;
}

#[tokio::test]
async fn after_a_sync_the_status_shows_the_run_and_the_mirrored_span() {
    let db = time_db("statussync").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![stopped_entry(1, "work")]);
    let app = time_app(&db, &fake);
    connect(&app).await;
    call(&app, "POST", "/api/time/sync", None).await;

    let reply = call(&app, "GET", "/api/time/status", None).await;

    assert_eq!(reply.json["time_entry_count"], 1);
    assert_eq!(reply.json["oldest_entry"], "2026-04-01T10:00:00Z");
    assert_eq!(reply.json["last_sync"]["outcome"], "success");
    assert_eq!(reply.json["last_sync"]["counts"]["inserted"], 1);
    db.drop().await;
}
