//! One access-log line per request, tagged with a request id.
//!
//! Layer order (outermost first): vet any incoming `x-request-id` -> assign
//! one if absent -> trace span + log
//! line -> echo the id on the response. The id is also in every log line
//! emitted while handling the request, so a browser error that shows the
//! response header can be found in the server log.

use axum::{
    Router,
    extract::{MatchedPath, Request},
    http::{HeaderName, Response, StatusCode},
    middleware::{self, Next},
};
use std::time::Duration;
use tower_http::{
    propagate_header::PropagateHeaderLayer,
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{Level, Span};
use vitarchy_core::level_for_status;

/// Header carrying the request id, in both directions.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// The frontend polls this every few seconds; logging each poll at `info`
/// would bury everything else.
const HEALTH_PATH: &str = "/api/health";

/// The route a request was served by, handed from the inner middleware to the
/// response logger (which sees only the response).
#[derive(Clone)]
struct LoggedPath(String);

/// Level for a finished request. Failures keep their severity even on the
/// health route (a 503 there is worth seeing); only healthy polls are quiet.
#[must_use]
pub fn level_for_response(path: &str, status: StatusCode) -> Level {
    match level_for_status(status) {
        Level::INFO if path == HEALTH_PATH => Level::DEBUG,
        level => level,
    }
}

/// Wraps `router` with request-id assignment, tracing and id propagation.
/// Applied once, in [`crate::app::build`].
pub fn layer(router: Router) -> Router {
    let header = HeaderName::from_static(REQUEST_ID_HEADER);
    router
        .layer(middleware::from_fn(record_matched_path))
        .layer(PropagateHeaderLayer::new(header.clone()))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(make_span)
                .on_request(())
                .on_response(log_response)
                .on_failure(()),
        )
        .layer(SetRequestIdLayer::new(header, MakeRequestUuid))
        .layer(middleware::from_fn(discard_unsafe_request_id))
}

/// Longest caller-supplied request id we accept.
const REQUEST_ID_MAX_LEN: usize = 64;

/// A caller-supplied id goes into every log line, so it must be short and
/// plain: 1-64 characters of `A-Za-z0-9._-`.
#[must_use]
pub fn is_acceptable_request_id(id: &str) -> bool {
    (1..=REQUEST_ID_MAX_LEN).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// Outermost middleware: drops an incoming id that fails
/// [`is_acceptable_request_id`], so the layer below generates a fresh UUID.
async fn discard_unsafe_request_id(mut request: Request, next: Next) -> axum::response::Response {
    let acceptable = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .is_none_or(|v| v.to_str().is_ok_and(is_acceptable_request_id));
    if !acceptable {
        request.headers_mut().remove(REQUEST_ID_HEADER);
    }
    next.run(request).await
}

/// Innermost middleware: records the matched route (falling back to the raw
/// path for unmatched requests, e.g. 404s) on the response for [`log_response`].
async fn record_matched_path(request: Request, next: Next) -> axum::response::Response {
    let path = request_path(&request);
    let mut response = next.run(request).await;
    response.extensions_mut().insert(LoggedPath(path));
    response
}

/// The matched route pattern (`/api/settings/modules/{id}`) keeps log
/// cardinality low; the raw path is used only when no route matched.
fn request_path(request: &Request) -> String {
    request
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| request.uri().path(), MatchedPath::as_str)
        .to_owned()
}

/// Opens the per-request span so every log line inside carries `request_id`.
fn make_span(request: &axum::http::Request<axum::body::Body>) -> Span {
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-");
    tracing::info_span!(
        "request",
        request_id,
        method = %request.method(),
        path = %request_path(request),
    )
}

/// Logs the one access line per request, at the level its status calls for.
fn log_response(response: &Response<axum::body::Body>, latency: Duration, _span: &Span) {
    let path = response
        .extensions()
        .get::<LoggedPath>()
        .map_or("", |p| p.0.as_str());
    let status = response.status();
    let (code, latency_ms) = (status.as_u16(), latency_ms(latency));
    // `tracing` macros need a constant level, hence one arm per level.
    match level_for_response(path, status) {
        Level::ERROR => tracing::error!(status = code, latency_ms, "response"),
        Level::WARN => tracing::warn!(status = code, latency_ms, "response"),
        Level::INFO => tracing::info!(status = code, latency_ms, "response"),
        _ => tracing::debug!(status = code, latency_ms, "response"),
    }
}

/// Milliseconds with microsecond precision (`1.234`), the unit the log field uses.
fn latency_ms(latency: Duration) -> f64 {
    let micros = latency.as_micros();
    // Precision loss only past 2^52 microseconds (over 140 years).
    #[allow(clippy::cast_precision_loss)]
    let micros = micros as f64;
    micros / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_ids_must_be_short_and_plain() {
        for ok in ["a", "abc-123_X.y", &"a".repeat(64)] {
            assert!(is_acceptable_request_id(ok), "{ok}");
        }
        for bad in ["", "a b", "a;b", "é", "a/b", &"a".repeat(65)] {
            assert!(!is_acceptable_request_id(bad), "{bad}");
        }
    }

    #[test]
    fn latency_is_reported_in_milliseconds_with_microsecond_precision() {
        assert!((latency_ms(Duration::from_micros(1_234)) - 1.234).abs() < 1e-9);
        assert!((latency_ms(Duration::from_secs(2)) - 2000.0).abs() < 1e-9);
        assert!(latency_ms(Duration::ZERO).abs() < 1e-9);
    }

    #[test]
    fn healthy_health_polls_are_debug() {
        assert_eq!(
            level_for_response("/api/health", StatusCode::OK),
            Level::DEBUG
        );
    }

    #[test]
    fn failing_health_polls_keep_their_severity() {
        assert_eq!(
            level_for_response("/api/health", StatusCode::SERVICE_UNAVAILABLE),
            Level::ERROR
        );
    }

    #[test]
    fn other_paths_follow_the_status() {
        assert_eq!(
            level_for_response("/api/settings", StatusCode::OK),
            Level::INFO
        );
        assert_eq!(
            level_for_response("/api/settings/core", StatusCode::BAD_REQUEST),
            Level::WARN
        );
        assert_eq!(
            level_for_response("/api/settings", StatusCode::INTERNAL_SERVER_ERROR),
            Level::ERROR
        );
    }
}
