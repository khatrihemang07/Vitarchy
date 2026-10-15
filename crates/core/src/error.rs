use axum::{
    Json,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

/// Error type for HTTP handlers. Renders as an RFC 9457 problem document.
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

/// JSON body of every error response (`application/problem+json`).
#[derive(Serialize, ToSchema)]
pub struct Problem {
    /// URI identifying the problem type; `about:blank` when the status says it all.
    #[serde(rename = "type")]
    pub kind: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
}

impl ApiError {
    fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) | Self::Database(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Database(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                StatusCode::CONFLICT
            }
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Client-safe description; internal failures are logged, never exposed.
    fn detail(&self) -> String {
        match (self.status(), self) {
            (StatusCode::INTERNAL_SERVER_ERROR, _) => "an internal error occurred".to_owned(),
            (_, Self::NotFound(d) | Self::BadRequest(d) | Self::Conflict(d)) => d.clone(),
            (StatusCode::NOT_FOUND, _) => "resource not found".to_owned(),
            _ => "resource already exists".to_owned(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        if status.is_server_error() {
            tracing::error!(error = ?self, "request failed");
        }
        let problem = Problem {
            kind: "about:blank".to_owned(),
            title: status.canonical_reason().unwrap_or("Error").to_owned(),
            status: status.as_u16(),
            detail: self.detail(),
        };
        let mut res = (status, Json(problem)).into_response();
        res.headers_mut().insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        res
    }
}
