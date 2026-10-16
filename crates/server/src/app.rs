//! Builds the HTTP application from the registered Modules.

use std::path::Path;

use axum::Router;
use utoipa::{OpenApi as _, openapi::OpenApi};
use utoipa_axum::router::OpenApiRouter;
use vitarchy_core::{ApiError, AppState, Module};

use crate::{health, openapi::ApiDoc, request_log, spa};

/// Mounts every Module at `/api/{id}` and merges their `OpenAPI` documents.
#[must_use]
pub fn compose(modules: Vec<Module>) -> OpenApiRouter<AppState> {
    modules.into_iter().fold(
        OpenApiRouter::with_openapi(ApiDoc::openapi()).merge(health::router()),
        |api, module| api.nest(&format!("/api/{}", module.id()), module.router()),
    )
}

/// Unknown `/api` paths are API errors (problem JSON), never the web app.
async fn api_not_found() -> ApiError {
    ApiError::NotFound("no such API route".to_owned())
}

/// Builds the full HTTP application, including `GET /api/openapi.json`.
/// With `web_dist`, the same server also serves the built web app.
pub fn build(state: AppState, modules: Vec<Module>, web_dist: Option<&Path>) -> Router {
    let (router, spec) = compose(modules).split_for_parts();
    let router = router
        .route(
            "/api/openapi.json",
            axum::routing::get(move || async move { axum::Json::<OpenApi>(spec) }),
        )
        .route("/api", axum::routing::any(api_not_found))
        .route("/api/{*rest}", axum::routing::any(api_not_found))
        .with_state(state);
    let router = match web_dist {
        Some(dist) => spa::mount(router, dist),
        None => router,
    };
    request_log::layer(router)
}
