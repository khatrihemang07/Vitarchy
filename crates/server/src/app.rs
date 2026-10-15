use axum::Router;
use utoipa::{OpenApi as _, openapi::OpenApi};
use utoipa_axum::router::OpenApiRouter;
use vitarchy_core::{AppState, Module};

use crate::{health, openapi::ApiDoc};

/// Mounts every Module at `/api/{id}` and merges their `OpenAPI` documents.
#[must_use]
pub fn compose(modules: Vec<Module>) -> OpenApiRouter<AppState> {
    modules.into_iter().fold(
        OpenApiRouter::with_openapi(ApiDoc::openapi()).merge(health::router()),
        |api, module| api.nest(&format!("/api/{}", module.id()), module.router()),
    )
}

/// Builds the full HTTP application, including `GET /api/openapi.json`.
pub fn build(state: AppState, modules: Vec<Module>) -> Router {
    let (router, spec) = compose(modules).split_for_parts();
    router
        .route(
            "/api/openapi.json",
            axum::routing::get(move || async move { axum::Json::<OpenApi>(spec) }),
        )
        .with_state(state)
}
