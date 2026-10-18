use crate::common::unreachable_pool;
use axum::{body::Body, http::Request};
use tower::ServiceExt;
use vitarchy_core::{AppState, log_capture::capture_logs};
use vitarchy_server::app;

async fn request_id_of(request: Request<Body>) -> String {
    let res = app::build(AppState::new(unreachable_pool()), vec![], None)
        .oneshot(request)
        .await
        .unwrap();
    res.headers()["x-request-id"].to_str().unwrap().to_owned()
}

fn is_uuid(id: &str) -> bool {
    id.len() == 36 && id.matches('-').count() == 4
}

#[tokio::test]
async fn every_response_carries_a_generated_uuid_request_id() {
    let id = request_id_of(
        Request::get("/api/openapi.json")
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(id.len(), 36, "{id}");
    assert_eq!(id.matches('-').count(), 4, "{id}");
}

#[tokio::test]
async fn unmatched_routes_also_get_a_request_id() {
    let id = request_id_of(Request::get("/nope").body(Body::empty()).unwrap()).await;

    assert!(!id.is_empty());
}

#[tokio::test]
async fn an_incoming_request_id_is_echoed_back() {
    let request = Request::get("/api/openapi.json")
        .header("x-request-id", "from-the-browser-42")
        .body(Body::empty())
        .unwrap();

    assert_eq!(request_id_of(request).await, "from-the-browser-42");
}

#[tokio::test]
async fn an_incoming_request_id_that_is_unsafe_to_log_is_replaced() {
    let too_long = "a".repeat(65);
    for bad in [
        "",
        "has space",
        "semi;colon",
        "line\tbreak",
        "quote\"",
        &too_long,
    ] {
        let request = Request::get("/api/openapi.json")
            .header("x-request-id", bad)
            .body(Body::empty())
            .unwrap();

        let id = request_id_of(request).await;

        assert!(is_uuid(&id), "{bad:?} was echoed as {id:?}");
    }
}

#[tokio::test]
async fn an_incoming_request_id_at_the_length_limit_is_kept() {
    let id = format!("{}._-Z9", "a".repeat(59));
    let request = Request::get("/api/openapi.json")
        .header("x-request-id", &id)
        .body(Body::empty())
        .unwrap();

    assert_eq!(request_id_of(request).await, id);
}

#[tokio::test]
async fn the_response_log_line_carries_the_request_id_method_and_path() {
    let request = Request::get("/api/openapi.json")
        .header("x-request-id", "trace-me-1")
        .body(Body::empty())
        .unwrap();

    let (_, logs) = capture_logs(request_id_of(request)).await;

    let line: serde_json::Value = logs
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .find(|l| l["fields"]["message"] == "response")
        .unwrap_or_else(|| panic!("no response line in {logs}"));
    let span = &line["spans"][0];
    assert_eq!(span["request_id"], "trace-me-1", "{line}");
    assert_eq!(span["method"], "GET", "{line}");
    assert_eq!(span["path"], "/api/openapi.json", "{line}");
    assert_eq!(line["fields"]["status"], 200, "{line}");
    assert!(line["fields"]["latency_ms"].is_number(), "{line}");
}
