//! The Module's error type: an [`ApiError`] or a failed read, rendered as an RFC 9457
//! problem by the routes. Never carries file content or passwords.

use axum::response::{IntoResponse, Response};
use vitarchy_core::ApiError;

use crate::{
    problem::{Refusal, problem_response},
    reader::ReaderError,
};

/// What can go wrong in a Finance handler or import step.
#[derive(Debug, thiserror::Error)]
pub enum FinanceError {
    /// Anything core already knows how to render (bad input, not found, database).
    #[error(transparent)]
    Api(#[from] ApiError),
    /// The statement-reader helper failed.
    #[error(transparent)]
    Reader(#[from] ReaderError),
    /// Storing or reading a Statement original failed.
    #[error("the Statement file store failed: {0}")]
    Files(#[from] std::io::Error),
    /// Another confirm is already running; only one may at a time.
    #[error("another confirm is already running")]
    ConfirmInProgress,
    /// The request is refused for a reason the person can act on.
    #[error("refused: {}", .0.code)]
    Refused(Refusal),
}

impl From<sqlx::Error> for FinanceError {
    fn from(error: sqlx::Error) -> Self {
        Self::Api(ApiError::Database(error))
    }
}

impl FinanceError {
    /// Stable label for `runs.error_code` and logs.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Reader(error) => error.code(),
            Self::Files(_) => "files",
            Self::ConfirmInProgress => "confirm_in_progress",
            Self::Refused(refusal) => refusal.code,
            Self::Api(ApiError::Database(_)) => "database",
            Self::Api(_) => "rejected",
        }
    }
}

impl From<Refusal> for FinanceError {
    fn from(refusal: Refusal) -> Self {
        Self::Refused(refusal)
    }
}

impl IntoResponse for FinanceError {
    fn into_response(self) -> Response {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Refused(r) => problem_response(r.status, r.code, r.detail),
            Self::ConfirmInProgress => problem_response(
                409,
                "confirm_in_progress",
                "Another confirm is running. Wait for it to finish.",
            ),
            Self::Reader(error) => {
                problem_response(502, error.code(), "The statement reader failed.")
            }
            Self::Files(error) => {
                tracing::error!(?error, "statement file store failed");
                problem_response(500, "files", "The Statement files could not be stored.")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FinanceError;
    use crate::reader::ReaderError;
    use vitarchy_core::ApiError;

    #[test]
    fn codes_are_stable() {
        assert_eq!(
            FinanceError::from(ReaderError::Timeout).code(),
            "reader_timeout"
        );
        assert_eq!(
            FinanceError::ConfirmInProgress.code(),
            "confirm_in_progress"
        );
        assert_eq!(
            FinanceError::from(std::io::Error::other("x")).code(),
            "files"
        );
        assert_eq!(
            FinanceError::from(ApiError::BadRequest("x".into())).code(),
            "rejected"
        );
        assert_eq!(
            FinanceError::from(sqlx::Error::RowNotFound).code(),
            "database"
        );
    }
}
