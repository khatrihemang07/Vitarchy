//! Storage: one JSON document per scope in `settings.entries`.

use serde_json::Value;
use sqlx::PgPool;

/// Scope under which the Core settings are stored; no Module may use this id.
pub const CORE_SCOPE: &str = "core";

/// One scope's stored document, or `None` if it was never saved.
#[tracing::instrument(level = "debug", skip_all, fields(scope = %scope))]
pub async fn get(pool: &PgPool, scope: &str) -> Result<Option<Value>, sqlx::Error> {
    sqlx::query_scalar("SELECT value FROM settings.entries WHERE scope = $1")
        .bind(scope)
        .fetch_optional(pool)
        .await
}

/// Every Module scope (everything except `core`), ordered by id.
#[tracing::instrument(level = "debug", skip_all)]
pub async fn list_modules(pool: &PgPool) -> Result<Vec<(String, Value)>, sqlx::Error> {
    sqlx::query_as("SELECT scope, value FROM settings.entries WHERE scope <> $1 ORDER BY scope")
        .bind(CORE_SCOPE)
        .fetch_all(pool)
        .await
}

/// Creates or replaces one scope's document and returns what it replaced
/// (`None` on the first save). One statement, so the write and the previous
/// value are atomic and a failure leaves nothing half-done.
#[tracing::instrument(level = "debug", skip_all, fields(scope = %scope))]
pub async fn upsert(
    pool: &PgPool,
    scope: &str,
    value: &Value,
) -> Result<Option<Value>, sqlx::Error> {
    // The CTE reads the row as it was before this statement's own write.
    sqlx::query_scalar(
        "WITH previous AS (SELECT value FROM settings.entries WHERE scope = $1)
         INSERT INTO settings.entries (scope, value) VALUES ($1, $2)
         ON CONFLICT (scope) DO UPDATE SET value = EXCLUDED.value, updated_at = now()
         RETURNING (SELECT value FROM previous)",
    )
    .bind(scope)
    .bind(value)
    .fetch_one(pool)
    .await
}
