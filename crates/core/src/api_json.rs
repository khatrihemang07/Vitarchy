use axum::extract::{FromRequest, Request, rejection::JsonRejection};
use serde::de::DeserializeOwned;

use crate::ApiError;

/// Drop-in replacement for `axum::Json` as a request extractor: malformed JSON,
/// a wrong content type or an invalid field value is rejected with the same
/// RFC 9457 problem document as every other [`ApiError`], instead of axum's
/// plain-text rejection.
pub struct ApiJson<T>(pub T);

impl<S, T> FromRequest<S> for ApiJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(axum::Json(value)) => Ok(Self(value)),
            Err(rejection) => Err(reject(&rejection)),
        }
    }
}

fn reject(rejection: &JsonRejection) -> ApiError {
    ApiError::BadRequest(rejection.body_text())
}
