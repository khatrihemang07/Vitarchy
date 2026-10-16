//! `PUT` and `DELETE /api/time/connection`: the Toggl connection.

use axum::{Extension, Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vitarchy_core::{ApiError, ApiJson, AppState, Problem};

use crate::{
    error::{ProblemBody, TimeError},
    repo::connection,
    toggl::api::TogglFactory,
};

// Deliberately not `Debug`: it holds the token.
/// Body of `PUT /api/time/connection`.
#[derive(Deserialize, ToSchema)]
pub struct ConnectRequest {
    /// The person's Toggl API token. Stored server-side, never returned.
    pub api_token: String,
}

/// Answer to a successful connect: what the token reached, never the token.
#[derive(Serialize, ToSchema)]
pub struct ConnectResponse {
    /// Name of the workspace that will be synced.
    pub workspace_name: String,
}

/// Checks the token with Toggl and stores it with the default workspace.
#[utoipa::path(
    put,
    path = "/connection",
    tag = "time",
    request_body = ConnectRequest,
    responses(
        (status = 200, description = "Connected", body = ConnectResponse),
        (status = 400, description = "Empty token", body = Problem),
        (status = 401, description = "Toggl rejected the token", body = ProblemBody),
        (status = 429, description = "Toggl request limit reached", body = ProblemBody),
        (status = 502, description = "Toggl unreachable or unreadable", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn put_connection(
    State(state): State<AppState>,
    Extension(factory): Extension<TogglFactory>,
    ApiJson(body): ApiJson<ConnectRequest>,
) -> Result<Json<ConnectResponse>, TimeError> {
    let token = body.api_token.trim();
    if token.is_empty() {
        return Err(ApiError::BadRequest("The API token is empty.".to_owned()).into());
    }
    let api = factory(token);
    let me = api.me().await?;
    let workspace = api.workspace(me.default_workspace_id).await?;
    connection::save(
        &state.pool,
        token,
        me.default_workspace_id,
        &workspace.name,
        me.id,
    )
    .await?;
    tracing::info!(workspace_id = me.default_workspace_id, "toggl connected");
    Ok(Json(ConnectResponse {
        workspace_name: workspace.name,
    }))
}

/// Forgets the token. Time Entries already synced are kept.
#[utoipa::path(
    delete,
    path = "/connection",
    tag = "time",
    responses((status = 204, description = "Disconnected (also when nothing was connected)"))
)]
#[tracing::instrument(skip_all)]
pub async fn delete_connection(State(state): State<AppState>) -> Result<StatusCode, TimeError> {
    let removed = connection::delete(&state.pool).await?;
    tracing::info!(removed, "toggl disconnected");
    Ok(StatusCode::NO_CONTENT)
}
