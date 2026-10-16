//! Which log level an HTTP status deserves: one rule for request logging and
//! error rendering, so they can never disagree.

use axum::http::StatusCode;
use tracing::Level;

/// The one rule that maps an HTTP status to a log level, shared by request
/// logging and error rendering so they can never disagree: server errors are
/// `error`, client errors `warn`, everything else `info`.
#[must_use]
pub fn level_for_status(status: StatusCode) -> Level {
    if status.is_server_error() {
        Level::ERROR
    } else if status.is_client_error() {
        Level::WARN
    } else {
        Level::INFO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_maps_to_level() {
        assert_eq!(level_for_status(StatusCode::OK), Level::INFO);
        assert_eq!(level_for_status(StatusCode::FOUND), Level::INFO);
        assert_eq!(level_for_status(StatusCode::NOT_FOUND), Level::WARN);
        assert_eq!(level_for_status(StatusCode::BAD_GATEWAY), Level::ERROR);
    }
}
