mod common;

use axum::{body::Body, http::Request};
use common::unreachable_pool;
use http_body_util::BodyExt;
use sqlx::PgPool;
use tower::ServiceExt;
use vitarchy_core::{AppState, test_db::TestDb};
use vitarchy_server::app;

async fn get_health(pool: PgPool) -> (u16, serde_json::Value) {
    let res = app::build(AppState::new(pool), vec![], None)
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status().as_u16();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    (status, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn reports_ok_when_database_is_reachable() {
    let db = TestDb::create("health").await;

    let (status, body) = get_health(db.pool.clone()).await;

    assert_eq!(status, 200);
    assert_eq!(body, serde_json::json!({"status": "ok", "db": "ok"}));
    db.drop().await;
}

#[tokio::test]
async fn reports_503_instead_of_panicking_when_database_is_down() {
    let (status, body) = get_health(unreachable_pool()).await;

    assert_eq!(status, 503);
    assert_eq!(
        body,
        serde_json::json!({"status": "degraded", "db": "down"})
    );
}

#[tokio::test]
async fn health_is_documented_in_openapi() {
    let spec = serde_json::to_value(app::compose(vec![]).get_openapi()).unwrap();
    assert!(spec["paths"]["/api/health"]["get"].is_object(), "{spec}");
}
