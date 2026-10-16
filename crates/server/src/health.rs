//! `GET /api/health`: liveness plus a database ping.

use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use vitarchy_core::AppState;

const DB_PING_TIMEOUT: Duration = Duration::from_secs(2);

/// Overall server health.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Everything works.
    Ok,
    /// The server runs but a dependency is down.
    Degraded,
}

/// Whether Postgres answered the ping.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum DbStatus {
    /// The ping succeeded in time.
    Ok,
    /// The ping failed or timed out.
    Down,
}

/// Body of `GET /api/health`.
#[derive(Serialize, ToSchema)]
pub struct Health {
    /// Overall status.
    pub status: Status,
    /// Database status.
    pub db: DbStatus,
}

/// The health route.
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(health))
}

/// Reports server and database health.
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "system",
    responses(
        (status = 200, description = "Server and database are healthy", body = Health),
        (status = 503, description = "Database is unreachable", body = Health),
    )
)]
async fn health(State(state): State<AppState>) -> (StatusCode, Json<Health>) {
    let ping = tokio::time::timeout(
        DB_PING_TIMEOUT,
        sqlx::query("SELECT 1").execute(&state.pool),
    );
    match ping.await {
        Ok(Ok(_)) => (
            StatusCode::OK,
            Json(Health {
                status: Status::Ok,
                db: DbStatus::Ok,
            }),
        ),
        failure => {
            tracing::warn!(?failure, "database health check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(Health {
                    status: Status::Degraded,
                    db: DbStatus::Down,
                }),
            )
        }
    }
}
