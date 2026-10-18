//! The real HTTP client, against a local stand-in for Toggl (never the real one).

use axum::{
    Router,
    extract::Query,
    http::{HeaderMap, StatusCode},
    routing::get,
};
use serde_json::{Value, json};
use std::collections::HashMap;
use vitarchy_time::toggl::{
    api::TogglApi, client::HttpToggl, error::TogglError, types::EntryQuery,
};

async fn serve(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

#[tokio::test]
async fn it_sends_basic_auth_with_the_token_and_reads_me() {
    let router = Router::new().route(
        "/me",
        get(|headers: HeaderMap| async move {
            // base64("tok:api_token")
            assert_eq!(headers["authorization"], "Basic dG9rOmFwaV90b2tlbg==");
            axum::Json(json!({"id": 5, "default_workspace_id": 7, "email": "x"}))
        }),
    );
    let client = HttpToggl::new("tok", &serve(router).await);

    let me = client.me().await.unwrap();

    assert_eq!((me.id, me.default_workspace_id), (5, 7));
}

#[tokio::test]
async fn a_since_query_sends_the_unix_time_and_keeps_raw_records() {
    let router = Router::new().route(
        "/me/time_entries",
        get(|Query(q): Query<HashMap<String, String>>| async move {
            assert_eq!(q["since"], "1000");
            assert!(!q.contains_key("start_date"));
            axum::Json(json!([{
                "id": 1, "start": "2026-04-01T10:00:00Z", "stop": "2026-04-01T11:00:00Z",
                "duration": 3600, "future_field": 42
            }]))
        }),
    );
    let client = HttpToggl::new("tok", &serve(router).await);

    let entries = client.time_entries(&EntryQuery::Since(1000)).await.unwrap();

    assert_eq!(entries[0].raw["future_field"], 42);
}

#[tokio::test]
async fn a_range_query_sends_start_and_end_dates() {
    let router = Router::new().route(
        "/me/time_entries",
        get(|Query(q): Query<HashMap<String, String>>| async move {
            assert_eq!(
                (q["start_date"].as_str(), q["end_date"].as_str()),
                ("2026-01-01", "2026-04-02")
            );
            axum::Json(Value::Null)
        }),
    );
    let client = HttpToggl::new("tok", &serve(router).await);
    let query = EntryQuery::Range {
        start_date: "2026-01-01".parse().unwrap(),
        end_date: "2026-04-02".parse().unwrap(),
    };

    assert!(client.time_entries(&query).await.unwrap().is_empty());
}

#[tokio::test]
async fn statuses_map_to_the_error_classes() {
    let router = Router::new()
        .route("/me", get(|| async { StatusCode::FORBIDDEN }))
        .route(
            "/workspaces/1",
            get(|| async {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    [("X-Toggl-Quota-Resets-In", "42")],
                )
            }),
        );
    let client = HttpToggl::new("tok", &serve(router).await);

    assert_eq!(client.me().await.unwrap_err(), TogglError::InvalidToken);
    assert_eq!(
        client.workspace(1).await.unwrap_err(),
        TogglError::RateLimited {
            resets_in_secs: Some(42)
        }
    );
}

#[tokio::test]
async fn an_unreachable_server_is_unreachable() {
    let client = HttpToggl::new("tok", "http://127.0.0.1:1");

    assert_eq!(client.me().await.unwrap_err(), TogglError::Unreachable);
}

#[tokio::test]
async fn an_unreadable_body_is_malformed() {
    let router = Router::new().route("/me", get(|| async { "not json" }));
    let client = HttpToggl::new("tok", &serve(router).await);

    assert_eq!(client.me().await.unwrap_err(), TogglError::Malformed);
}

#[tokio::test]
async fn projects_are_read_page_by_page_until_a_short_page() {
    let router = Router::new().route(
        "/workspaces/1/projects",
        get(|Query(q): Query<HashMap<String, String>>| async move {
            assert_eq!(q["per_page"], "200");
            let page: u32 = q["page"].parse().unwrap();
            let (first, count) = match page {
                1 => (1, 200),
                2 => (201, 200),
                3 => (401, 5),
                _ => panic!("asked past the last page: {page}"),
            };
            let projects: Vec<Value> = (first..first + count)
                .map(|id| json!({"id": id, "name": "p"}))
                .collect();
            axum::Json(projects)
        }),
    );
    let client = HttpToggl::new("tok", &serve(router).await);

    let projects = client.projects(1).await.unwrap();

    assert_eq!(projects.len(), 405);
    assert_eq!(projects.last().unwrap().item.id, 405);
}
