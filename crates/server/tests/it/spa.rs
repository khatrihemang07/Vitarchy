use std::path::PathBuf;

use crate::common::unreachable_pool;
use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use tower::ServiceExt;
use vitarchy_core::AppState;
use vitarchy_server::{app, spa};

/// A fake web build in a unique temp dir; removed on drop.
struct FakeDist(PathBuf);

impl FakeDist {
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("vitarchy_spa_{label}_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::write(dir.join("index.html"), "<!doctype html><title>SPA</title>").unwrap();
        std::fs::write(dir.join("assets/app.js"), "console.log(1)").unwrap();
        Self(dir)
    }
}

impl Drop for FakeDist {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).ok();
    }
}

fn serving(dist: &FakeDist) -> Router {
    app::build(AppState::new(unreachable_pool()), vec![], Some(&dist.0))
}

async fn get(app: Router, uri: &str) -> (u16, String, String) {
    let res = app
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status().as_u16();
    let content_type = res
        .headers()
        .get("content-type")
        .map(|v| v.to_str().unwrap().to_owned())
        .unwrap_or_default();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        content_type,
        String::from_utf8_lossy(&body).into_owned(),
    )
}

#[tokio::test]
async fn serves_index_html_at_the_root() {
    let dist = FakeDist::new("root");
    let (status, content_type, body) = get(serving(&dist), "/").await;
    assert_eq!(status, 200);
    assert!(content_type.starts_with("text/html"), "{content_type}");
    assert!(body.contains("<title>SPA</title>"));
}

#[tokio::test]
async fn serves_built_assets_as_files() {
    let dist = FakeDist::new("asset");
    let (status, content_type, body) = get(serving(&dist), "/assets/app.js").await;
    assert_eq!(status, 200);
    assert!(content_type.contains("javascript"), "{content_type}");
    assert_eq!(body, "console.log(1)");
}

#[tokio::test]
async fn client_routes_fall_back_to_index_html() {
    let dist = FakeDist::new("fallback");
    let (status, _, body) = get(serving(&dist), "/settings/general").await;
    assert_eq!(status, 200);
    assert!(body.contains("<title>SPA</title>"));
}

#[tokio::test]
async fn api_routes_keep_priority_over_the_web_app() {
    let dist = FakeDist::new("health");
    let (status, content_type, body) = get(serving(&dist), "/api/health").await;
    // The unreachable database makes this 503, but it is the API's JSON, not index.html.
    assert_eq!(status, 503);
    assert!(
        content_type.starts_with("application/json"),
        "{content_type}"
    );
    assert!(body.contains("degraded"));
}

#[tokio::test]
async fn unknown_api_paths_are_problem_json_404s_not_the_web_app() {
    let dist = FakeDist::new("api404");
    for uri in ["/api/nope", "/api/settings/nope/deeper", "/api"] {
        let (status, content_type, body) = get(serving(&dist), uri).await;
        assert_eq!(status, 404, "{uri}");
        assert!(
            content_type.starts_with("application/problem+json"),
            "{uri}: {content_type}"
        );
        assert!(!body.contains("<title>"), "{uri}");
    }
}

#[tokio::test]
async fn unknown_api_paths_are_problem_json_404s_without_a_web_build_too() {
    let app = app::build(AppState::new(unreachable_pool()), vec![], None);
    let (status, content_type, _) = get(app, "/api/nope").await;
    assert_eq!(status, 404);
    assert!(
        content_type.starts_with("application/problem+json"),
        "{content_type}"
    );
}

#[test]
fn ensure_built_rejects_a_dir_without_index_html() {
    let empty = std::env::temp_dir();
    let err = spa::ensure_built(&empty.join("vitarchy_definitely_missing")).unwrap_err();
    assert!(err.to_string().contains("make web-build"), "{err}");
    let dist = FakeDist::new("built");
    assert!(spa::ensure_built(&dist.0).is_ok());
}

#[tokio::test]
async fn missing_built_assets_are_404_not_index_html() {
    let dist = FakeDist::new("assets404");
    for uri in ["/assets/missing.js", "/assets/nested/dir"] {
        let (status, _, body) = get(serving(&dist), uri).await;
        assert_eq!(status, 404, "{uri}");
        assert!(!body.contains("<title>"), "{uri}");
    }
}

#[tokio::test]
async fn missing_files_with_an_extension_are_404() {
    let dist = FakeDist::new("ext404");
    for uri in ["/favicon.png", "/settings/report.pdf"] {
        let (status, _, body) = get(serving(&dist), uri).await;
        assert_eq!(status, 404, "{uri}");
        assert!(!body.contains("<title>"), "{uri}");
    }
}
