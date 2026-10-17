//! The Module's error type: an [`ApiError`] or a failed Toggl call, rendered as
//! an RFC 9457 problem. Toggl failures need a machine `code` (and a reset time)
//! the web app acts on, which core's domain-free `Problem` does not carry.

use axum::{
    Json,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;
use vitarchy_core::ApiError;

use crate::{activity::ActivityError, toggl::error::TogglError};

/// What can go wrong in a Time handler.
#[derive(Debug, thiserror::Error)]
pub enum TimeError {
    /// Anything core already knows how to render (bad input, not found, database).
    #[error(transparent)]
    Api(#[from] ApiError),
    /// Toggl refused or could not be reached.
    #[error(transparent)]
    Toggl(#[from] TogglError),
    /// Another Sync is already running; only one may at a time.
    #[error("a Sync is already running")]
    SyncInProgress,
    /// The Toggl connection was replaced while this Sync was running.
    #[error("the Toggl connection changed during the Sync")]
    ConnectionChanged,
    /// The Activity Snapshot is missing or unreadable.
    #[error(transparent)]
    Activity(#[from] ActivityError),
}

impl From<sqlx::Error> for TimeError {
    fn from(error: sqlx::Error) -> Self {
        Self::Api(ApiError::Database(error))
    }
}

impl TimeError {
    /// Stable label for `runs.error_code`.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Toggl(error) => error.code(),
            Self::Activity(error) => error.code(),
            Self::SyncInProgress => "sync_in_progress",
            Self::ConnectionChanged => "connection_changed",
            Self::Api(ApiError::Database(_)) => "database",
            Self::Api(_) => "rejected",
        }
    }
}

/// Problem body for a failed Toggl call: core's problem fields plus a `code`.
#[derive(Serialize, ToSchema)]
pub struct ProblemBody {
    /// Always `about:blank`; the status and code say it all.
    #[serde(rename = "type")]
    pub kind: String,
    /// The status's reason phrase.
    pub title: String,
    /// The HTTP status.
    pub status: u16,
    /// Explanation fit to show the person; never holds the token.
    pub detail: String,
    /// `invalid_token`, `rate_limited`, `unreachable`, `unexpected`, `malformed`,
    /// `sync_in_progress`, `connection_changed`, `no_snapshot` or `snapshot_unreadable`.
    pub code: String,
    /// For `rate_limited`: seconds until Toggl accepts requests again.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resets_in_secs: Option<u64>,
}

impl ProblemBody {
    fn for_activity(error: ActivityError) -> Self {
        let (status, detail) = match error {
            ActivityError::NoSnapshot => (
                StatusCode::NOT_FOUND,
                "There is no Snapshot yet. Install the helper to take one.",
            ),
            ActivityError::Unreadable => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "The Snapshot could not be read.",
            ),
        };
        Self::new(status, detail, error.code(), None)
    }

    fn new(status: StatusCode, detail: &str, code: &str, resets_in_secs: Option<u64>) -> Self {
        Self {
            kind: "about:blank".to_owned(),
            title: status.canonical_reason().unwrap_or("Error").to_owned(),
            status: status.as_u16(),
            detail: detail.to_owned(),
            code: code.to_owned(),
            resets_in_secs,
        }
    }

    fn for_toggl(error: &TogglError) -> Self {
        let (status, detail) = match error {
            TogglError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "Toggl rejected this API token.")
            }
            TogglError::RateLimited { .. } => (
                StatusCode::TOO_MANY_REQUESTS,
                "Toggl's request limit was reached. Try again later.",
            ),
            TogglError::Unreachable => (StatusCode::BAD_GATEWAY, "Could not reach Toggl."),
            TogglError::Unexpected(_) => (StatusCode::BAD_GATEWAY, "Toggl answered unexpectedly."),
            TogglError::Malformed => (
                StatusCode::BAD_GATEWAY,
                "Toggl sent a response Vitarchy could not read.",
            ),
        };
        let resets_in_secs = match error {
            TogglError::RateLimited { resets_in_secs } => *resets_in_secs,
            _ => None,
        };
        Self::new(status, detail, error.code(), resets_in_secs)
    }
}

impl IntoResponse for TimeError {
    fn into_response(self) -> Response {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Toggl(error) => problem_response(ProblemBody::for_toggl(&error)),
            Self::Activity(error) => problem_response(ProblemBody::for_activity(error)),
            Self::SyncInProgress => problem_response(ProblemBody::new(
                StatusCode::CONFLICT,
                "A Sync is already running. Wait for it to finish.",
                "sync_in_progress",
                None,
            )),
            Self::ConnectionChanged => problem_response(ProblemBody::new(
                StatusCode::CONFLICT,
                "Toggl was connected again while syncing, so nothing was saved. Sync again.",
                "connection_changed",
                None,
            )),
        }
    }
}

/// Renders a coded problem, logging it once where it is rendered.
fn problem_response(body: ProblemBody) -> Response {
    tracing::warn!(status = body.status, code = %body.code, "time request failed");
    let status = StatusCode::from_u16(body.status).unwrap_or(StatusCode::BAD_GATEWAY);
    let mut response = (status, Json(body)).into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/problem+json"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_toggl_failure_has_its_own_status_and_code() {
        let cases = [
            (TogglError::InvalidToken, 401, "invalid_token"),
            (
                TogglError::RateLimited {
                    resets_in_secs: Some(60),
                },
                429,
                "rate_limited",
            ),
            (TogglError::Unreachable, 502, "unreachable"),
        ];
        for (error, status, code) in cases {
            let body = ProblemBody::for_toggl(&error);
            assert_eq!((body.status, body.code.as_str()), (status, code));
        }
    }

    #[test]
    fn only_a_rate_limit_carries_a_reset_time() {
        let limited = ProblemBody::for_toggl(&TogglError::RateLimited {
            resets_in_secs: Some(60),
        });
        assert_eq!(limited.resets_in_secs, Some(60));
        assert_eq!(
            ProblemBody::for_toggl(&TogglError::Unreachable).resets_in_secs,
            None
        );
    }

    #[test]
    fn sync_conflicts_are_409s_with_their_own_codes() {
        assert_eq!(TimeError::SyncInProgress.code(), "sync_in_progress");
        assert_eq!(TimeError::ConnectionChanged.code(), "connection_changed");
        let response = TimeError::SyncInProgress.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn a_failed_run_is_labelled_by_its_cause() {
        assert_eq!(
            TimeError::Toggl(TogglError::InvalidToken).code(),
            "invalid_token"
        );
        assert_eq!(
            TimeError::from(sqlx::Error::PoolTimedOut).code(),
            "database"
        );
    }
}
