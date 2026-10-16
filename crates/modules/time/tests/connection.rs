//! Connecting and disconnecting Toggl.

mod common;

use common::{FakeToggl, TOKEN, call, connect, stopped_entry, time_app, time_db};
use serde_json::json;

#[tokio::test]
async fn a_valid_token_connects_and_names_the_workspace() {
    let db = time_db("connok").await;
    let app = time_app(&db, &FakeToggl::new());

    let body = json!({"api_token": TOKEN}).to_string();
    let reply = call(&app, "PUT", "/api/time/connection", Some(&body)).await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    assert_eq!(reply.json["workspace_name"], "My Workspace");
    assert!(!reply.text.contains(TOKEN));
    let status = call(&app, "GET", "/api/time/status", None).await;
    assert_eq!(status.json["connected"], true);
    assert_eq!(status.json["workspace_name"], "My Workspace");
    db.drop().await;
}

#[tokio::test]
async fn an_invalid_token_is_refused_and_nothing_is_stored() {
    let db = time_db("connbad").await;
    let app = time_app(&db, &FakeToggl::new());

    let reply = call(
        &app,
        "PUT",
        "/api/time/connection",
        Some(r#"{"api_token":"nope"}"#),
    )
    .await;

    assert_eq!(reply.status, 401);
    assert_eq!(reply.content_type, "application/problem+json");
    assert_eq!(reply.json["code"], "invalid_token");
    let status = call(&app, "GET", "/api/time/status", None).await;
    assert_eq!(status.json["connected"], false);
    db.drop().await;
}

#[tokio::test]
async fn an_empty_token_is_a_bad_request() {
    let db = time_db("connempty").await;
    let app = time_app(&db, &FakeToggl::new());

    let reply = call(
        &app,
        "PUT",
        "/api/time/connection",
        Some(r#"{"api_token":"  "}"#),
    )
    .await;

    assert_eq!(reply.status, 400);
    db.drop().await;
}

#[tokio::test]
async fn disconnecting_removes_the_token_but_keeps_the_synced_entries() {
    let db = time_db("disconnect").await;
    let fake = FakeToggl::new();
    fake.with(|w| w.entries = vec![stopped_entry(1, "kept")]);
    let app = time_app(&db, &fake);
    connect(&app).await;
    call(&app, "POST", "/api/time/sync", None).await;

    let reply = call(&app, "DELETE", "/api/time/connection", None).await;

    assert_eq!(reply.status, 204);
    let status = call(&app, "GET", "/api/time/status", None).await;
    assert_eq!(status.json["connected"], false);
    assert_eq!(status.json["time_entry_count"], 1);
    let tokens: i64 = sqlx::query_scalar("SELECT count(*) FROM time.connection")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(tokens, 0);
    db.drop().await;
}

#[tokio::test]
async fn disconnecting_when_not_connected_still_succeeds() {
    let db = time_db("disconnectnone").await;
    let app = time_app(&db, &FakeToggl::new());

    assert_eq!(
        call(&app, "DELETE", "/api/time/connection", None)
            .await
            .status,
        204
    );
    db.drop().await;
}
