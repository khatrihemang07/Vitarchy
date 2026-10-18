//! The request id, for code that must hand it on (e.g. to a helper process).

use axum::{
    extract::FromRequestParts,
    http::{header::HeaderName, request::Parts},
};
use std::convert::Infallible;

/// Header carrying the request id, in both directions.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// The current request's id, as assigned by the server's request-id layer. `None` only when
/// the layer is absent (e.g. a router built without it in tests).
pub struct RequestId(pub Option<String>);

impl<S: Send + Sync> FromRequestParts<S> for RequestId {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let name = HeaderName::from_static(REQUEST_ID_HEADER);
        Ok(Self(
            parts
                .headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_owned),
        ))
    }
}

#[cfg(test)]
mod tests {
    use axum::{extract::FromRequestParts, http::Request};

    use super::RequestId;

    async fn extract(request: Request<()>) -> Option<String> {
        let (mut parts, ()) = request.into_parts();
        RequestId::from_request_parts(&mut parts, &())
            .await
            .unwrap()
            .0
    }

    #[tokio::test]
    async fn reads_the_header() {
        let request = Request::builder()
            .header("x-request-id", "abc-1")
            .body(())
            .unwrap();
        assert_eq!(extract(request).await.as_deref(), Some("abc-1"));
    }

    #[tokio::test]
    async fn is_none_without_the_header() {
        assert_eq!(extract(Request::new(())).await, None);
    }
}
