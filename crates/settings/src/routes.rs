//! `/api/settings`: read everything, write the core scope, write a Module scope.

use std::collections::BTreeMap;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
use serde_json::{Map, Value};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use vitarchy_core::{ApiError, ApiJson, AppState, Problem};

use crate::{
    core_settings::CoreSettings,
    repo::{self, CORE_SCOPE},
};

const MODULE_ID_MAX_LEN: usize = 32;

#[derive(Serialize, ToSchema)]
pub struct AllSettings {
    pub core: CoreSettings,
    /// Each Module's settings object, keyed by Module id.
    #[schema(value_type = HashMap<String, Object>)]
    pub modules: BTreeMap<String, Value>,
}

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(get_all))
        .routes(routes!(put_core))
        .routes(routes!(put_module))
}

/// All settings, with defaults filled in for anything not yet saved.
#[utoipa::path(
    get,
    path = "/",
    tag = "settings",
    responses(
        (status = 200, description = "Core and Module settings", body = AllSettings),
        (status = 500, description = "Internal error", body = Problem),
    )
)]
async fn get_all(State(state): State<AppState>) -> Result<Json<AllSettings>, ApiError> {
    let core = match repo::get(&state.pool, CORE_SCOPE).await? {
        // Lenient per field: one value that no longer parses resets only itself.
        Some(stored) => CoreSettings::from_stored(&stored),
        None => CoreSettings::default(),
    };
    let modules = repo::list_modules(&state.pool).await?.into_iter().collect();
    Ok(Json(AllSettings { core, modules }))
}

/// Replaces the core settings.
#[utoipa::path(
    put,
    path = "/core",
    tag = "settings",
    request_body = CoreSettings,
    responses(
        (status = 200, description = "The saved core settings", body = CoreSettings),
        (status = 400, description = "Invalid settings", body = Problem),
    )
)]
async fn put_core(
    State(state): State<AppState>,
    ApiJson(body): ApiJson<CoreSettings>,
) -> Result<Json<CoreSettings>, ApiError> {
    let settings = body.validated()?;
    let stored =
        serde_json::to_value(&settings).map_err(|e| ApiError::BadRequest(e.to_string()))?;
    repo::upsert(&state.pool, CORE_SCOPE, &stored).await?;
    Ok(Json(settings))
}

/// Replaces one Module's settings. The shape is the Module's own business.
#[utoipa::path(
    put,
    path = "/modules/{id}",
    tag = "settings",
    params(("id" = String, Path, description = "Module id: lowercase letters, digits and dashes, at most 32 characters")),
    request_body(content = Object, description = "The Module's settings object"),
    responses(
        (status = 200, description = "The saved settings object", body = Object),
        (status = 400, description = "Invalid Module id or body is not a JSON object", body = Problem),
    )
)]
async fn put_module(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ApiJson(body): ApiJson<Value>,
) -> Result<Json<Map<String, Value>>, ApiError> {
    if !is_valid_module_id(&id) {
        return Err(ApiError::BadRequest(format!(
            "module id must match ^[a-z][a-z0-9-]{{0,{}}}$",
            MODULE_ID_MAX_LEN - 1
        )));
    }
    let Value::Object(settings) = body else {
        return Err(ApiError::BadRequest(
            "module settings must be a JSON object".to_owned(),
        ));
    };
    repo::upsert(&state.pool, &id, &Value::Object(settings.clone())).await?;
    Ok(Json(settings))
}

/// `^[a-z][a-z0-9-]{0,31}$`. "core" is reserved for the core scope.
fn is_valid_module_id(id: &str) -> bool {
    let mut chars = id.chars();
    id.len() <= MODULE_ID_MAX_LEN
        && id != CORE_SCOPE
        && chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
