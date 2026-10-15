use axum::{Router, body::Body, http::Request, routing::post};
use http_body_util::BodyExt;
use serde::Deserialize;
use tower::ServiceExt;
use vitarchy_core::ApiJson;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Color {
    Red,
}

#[derive(Deserialize)]
struct Paint {
    color: Color,
}

async fn paint(ApiJson(p): ApiJson<Paint>) -> &'static str {
    match p.color {
        Color::Red => "red",
    }
}

async fn send(content_type: Option<&str>, body: &str) -> (u16, String, Vec<u8>) {
    let mut req = Request::post("/");
    if let Some(ct) = content_type {
        req = req.header("content-type", ct);
    }
    let res = Router::new()
        .route("/", post(paint))
        .oneshot(req.body(Body::from(body.to_owned())).unwrap())
        .await
        .unwrap();
    let status = res.status().as_u16();
    let ct = res.headers()["content-type"].to_str().unwrap().to_owned();
    (
        status,
        ct,
        res.into_body().collect().await.unwrap().to_bytes().to_vec(),
    )
}

#[tokio::test]
async fn valid_body_is_extracted() {
    let (status, _, body) = send(Some("application/json"), r#"{"color":"red"}"#).await;
    assert_eq!((status, &body[..]), (200, &b"red"[..]));
}

#[tokio::test]
async fn bad_enum_value_is_a_400_problem() {
    let (status, ct, body) = send(Some("application/json"), r#"{"color":"teal"}"#).await;
    assert_eq!(status, 400);
    assert_eq!(ct, "application/problem+json");
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["title"], "Bad Request");
    assert!(body["detail"].as_str().unwrap().contains("teal"), "{body}");
}

#[tokio::test]
async fn malformed_json_and_missing_content_type_are_400_problems() {
    for (ct, raw) in [
        (Some("application/json"), "{nope"),
        (None, r#"{"color":"red"}"#),
    ] {
        let (status, ct, _) = send(ct, raw).await;
        assert_eq!((status, ct.as_str()), (400, "application/problem+json"));
    }
}
