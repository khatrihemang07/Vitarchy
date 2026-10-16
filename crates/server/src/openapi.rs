//! The `OpenAPI` document: the contract the web client is generated from.

use utoipa::{OpenApi, openapi::OpenApi as OpenApiDoc};
use vitarchy_core::Module;

use crate::app;

/// Document-level metadata; paths are merged in per Module.
#[derive(OpenApi)]
#[openapi(info(
    title = "Vitarchy API",
    description = "Personal life-management server. Each Module is mounted under /api/{module}."
))]
pub struct ApiDoc;

/// The merged `OpenAPI` document. Needs no database.
#[must_use]
pub fn spec(modules: Vec<Module>) -> OpenApiDoc {
    app::compose(modules).get_openapi().clone()
}
