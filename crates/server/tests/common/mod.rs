//! Shared test fixtures: a dummy Module and helpers.
#![allow(dead_code)]

use axum::Json;
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use vitarchy_core::{AppState, Module};

#[derive(Serialize, ToSchema)]
pub struct Pong {
    pub pong: bool,
}

/// Ping
#[utoipa::path(get, path = "/ping", responses((status = 200, body = Pong)))]
async fn ping() -> Json<Pong> {
    Json(Pong { pong: true })
}

fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(ping))
}

/// A Module that lives only in tests; owns the `dummy` schema.
pub fn dummy_module() -> Module {
    Module::new("dummy", router, sqlx::migrate!("./tests/dummy_migrations"))
}

/// A pool that never connects until used, pointing at a closed port.
pub fn unreachable_pool() -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(300))
        .connect_lazy("postgres://nobody:nothing@127.0.0.1:1/none")
        .expect("lazy pool")
}

/// `DATABASE_URL` from the environment or the repo's `.env`.
pub fn database_url() -> String {
    dotenvy::dotenv().ok();
    std::env::var("DATABASE_URL").expect("DATABASE_URL must be set (see .env.example)")
}
