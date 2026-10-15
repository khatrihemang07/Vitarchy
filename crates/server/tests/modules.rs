mod common;

use axum::{body::Body, http::Request};
use common::{dummy_module, unreachable_pool};
use http_body_util::BodyExt;
use tower::ServiceExt;
use vitarchy_core::AppState;
use vitarchy_server::app;

#[tokio::test]
async fn module_router_is_mounted_under_api_id() {
    let router = app::build(AppState::new(unreachable_pool()), vec![dummy_module()]);

    let res = router
        .oneshot(Request::get("/api/dummy/ping").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], br#"{"pong":true}"#);
}

#[tokio::test]
async fn module_paths_appear_in_merged_openapi_document() {
    let router = app::build(AppState::new(unreachable_pool()), vec![dummy_module()]);

    let res = router
        .oneshot(
            Request::get("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let spec: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(
        spec["paths"]["/api/dummy/ping"]["get"].is_object(),
        "{spec}"
    );
}
