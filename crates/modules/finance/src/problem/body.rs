//! Rendering a coded problem as an RFC 9457 response.

use axum::{
    Json,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

/// Problem body for a coded failure.
#[derive(Serialize, ToSchema)]
pub struct ProblemBody {
    /// Always `about:blank`; the status and code say it all.
    #[serde(rename = "type")]
    pub kind: String,
    /// The status's reason phrase.
    pub title: String,
    /// The HTTP status.
    pub status: u16,
    /// Explanation fit to show the person; never holds a password or file content.
    pub detail: String,
    /// A stable `snake_case` reason, e.g. `statement_not_readable` or `confirm_in_progress`.
    pub code: String,
}

/// Renders a coded problem, logging it once where it is rendered.
pub fn problem_response(status: u16, code: &str, detail: &str) -> Response {
    tracing::warn!(status, code, "finance request failed");
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let body = ProblemBody {
        kind: "about:blank".to_owned(),
        title: status.canonical_reason().unwrap_or("Error").to_owned(),
        status: status.as_u16(),
        detail: detail.to_owned(),
        code: code.to_owned(),
    };
    let mut response = (status, Json(body)).into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );
    response
}
