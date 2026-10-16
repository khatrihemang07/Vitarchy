//! Test fixtures: a stage database with the settings migrations applied,
//! and a request helper that talks to the real router.
#![allow(dead_code)]

use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use tower::ServiceExt;
use vitarchy_core::{AppState, test_db::TestDb};

/// A throwaway stage database with the settings migrations applied.
pub async fn settings_db(label: &str) -> TestDb {
    let db = TestDb::create(&format!("settings_{label}")).await;
    vitarchy_settings::feature()
        .migrator()
        .run(&db.pool)
        .await
        .unwrap();
    db
}

/// The settings routes over `db`, mounted as the server mounts them.
pub fn settings_app(db: &TestDb) -> Router {
    let (router, _) = utoipa_axum::router::OpenApiRouter::new()
        .nest("/api/settings", vitarchy_settings::feature().router())
        .split_for_parts();
    router.with_state(AppState::new(db.pool.clone()))
}

pub struct Reply {
    pub status: u16,
    pub content_type: String,
    pub json: serde_json::Value,
}

pub async fn call(app: &Router, method: &str, uri: &str, body: Option<&str>) -> Reply {
    let mut req = Request::builder().method(method).uri(uri);
    if body.is_some() {
        req = req.header("content-type", "application/json");
    }
    let res = app
        .clone()
        .oneshot(req.body(Body::from(body.unwrap_or("").to_owned())).unwrap())
        .await
        .unwrap();
    let status = res.status().as_u16();
    let content_type = res
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_owned())
        .unwrap_or_default();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    Reply {
        status,
        content_type,
        json: serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
    }
}
