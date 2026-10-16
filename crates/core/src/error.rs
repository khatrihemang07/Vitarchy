use axum::{
    Json,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use tracing::Level;
use utoipa::ToSchema;

use crate::log_level::level_for_status;

/// Error type for HTTP handlers. Renders as an RFC 9457 problem document.
#[derive(Debug, Error)]
pub enum ApiError {
    /// 404: the thing asked for does not exist.
    #[error("not found: {0}")]
    NotFound(String),
    /// 400: the request was malformed or held an invalid value.
    #[error("bad request: {0}")]
    BadRequest(String),
    /// 409: the request clashes with existing state.
    #[error("conflict: {0}")]
    Conflict(String),
    /// A database failure; mapped to 404, 409 or 500 by its kind.
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

/// JSON body of every error response (`application/problem+json`).
#[derive(Serialize, ToSchema)]
pub struct Problem {
    /// URI identifying the problem type; `about:blank` when the status says it all.
    #[serde(rename = "type")]
    pub kind: String,
    /// Short, human-readable summary: the status's reason phrase.
    pub title: String,
    /// The HTTP status code, repeated in the body.
    pub status: u16,
    /// Explanation for the caller. For 404/400/409 it is the message the handler
    /// chose (which may quote the rejected input, e.g. a bad enum value); for
    /// 5xx and database failures it is generic, never the internal error.
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

    /// Fixed, value-free label for a client error, safe to log. Messages can
    /// echo what the person entered (serde names the rejected value), so 4xx
    /// logs carry this instead.
    fn kind(status: StatusCode) -> &'static str {
        match status {
            StatusCode::NOT_FOUND => "not_found",
            StatusCode::CONFLICT => "conflict",
            _ => "bad_request",
        }
    }

    /// Logs the failure once, where it is rendered. 5xx carries the full
    /// source error (internal, hidden from the client). 4xx logs only status and
    /// kind: the message may hold entered values.
    fn log(&self, status: StatusCode) {
        if level_for_status(status) == Level::ERROR {
            tracing::error!(status = status.as_u16(), error = ?self, "request failed");
        } else {
            tracing::warn!(
                status = status.as_u16(),
                kind = Self::kind(status),
                "request rejected"
            );
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        self.log(status);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log_capture::capture_logs;

    async fn logged(error: ApiError) -> String {
        capture_logs(async { error.into_response() }).await.1
    }

    #[tokio::test]
    async fn a_rejected_request_logs_status_and_kind_but_not_the_message() {
        let out = logged(ApiError::BadRequest("unknown variant `purple`".into())).await;

        assert!(out.contains("\"WARN\""), "{out}");
        assert!(out.contains("\"status\":400"), "{out}");
        assert!(out.contains("bad_request"), "{out}");
        assert!(!out.contains("purple"), "{out}");
    }

    #[tokio::test]
    async fn every_client_error_has_its_own_kind_label() {
        let not_found = logged(ApiError::NotFound("secret-thing".into())).await;
        let conflict = logged(ApiError::Conflict("secret-thing".into())).await;

        assert!(not_found.contains("not_found") && !not_found.contains("secret-thing"));
        assert!(conflict.contains("conflict") && !conflict.contains("secret-thing"));
    }

    #[tokio::test]
    async fn a_server_failure_logs_its_source_error_at_error_level() {
        let out = logged(ApiError::Database(sqlx::Error::PoolTimedOut)).await;

        assert!(out.contains("\"ERROR\""), "{out}");
        assert!(out.contains("PoolTimedOut"), "{out}");
    }
}
