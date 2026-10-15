use axum::response::IntoResponse;
use http_body_util::BodyExt;
use vitarchy_core::ApiError;

async fn render(err: ApiError) -> (u16, String, serde_json::Value) {
    let res = err.into_response();
    let status = res.status().as_u16();
    let content_type = res.headers()["content-type"].to_str().unwrap().to_owned();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    (status, content_type, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn not_found_renders_a_404_problem_document() {
    let (status, content_type, body) = render(ApiError::NotFound("task 7".into())).await;

    assert_eq!(status, 404);
    assert_eq!(content_type, "application/problem+json");
    assert_eq!(
        body,
        serde_json::json!({
            "type": "about:blank",
            "title": "Not Found",
            "status": 404,
            "detail": "task 7"
        })
    );
}

#[tokio::test]
async fn bad_request_renders_a_400_problem_document() {
    let (status, _, body) = render(ApiError::BadRequest("amount must be positive".into())).await;

    assert_eq!(status, 400);
    assert_eq!(body["title"], "Bad Request");
    assert_eq!(body["detail"], "amount must be positive");
}

#[tokio::test]
async fn missing_row_maps_to_404() {
    let (status, _, _) = render(sqlx::Error::RowNotFound.into()).await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn unexpected_database_errors_are_500_and_do_not_leak_details() {
    let (status, _, body) = render(sqlx::Error::PoolTimedOut.into()).await;

    assert_eq!(status, 500);
    assert_eq!(body["title"], "Internal Server Error");
    assert!(!body["detail"].as_str().unwrap().contains("pool"), "{body}");
}
