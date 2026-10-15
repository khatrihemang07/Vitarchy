//! Storage: one JSON document per scope in `settings.entries`.

use serde_json::Value;
use sqlx::PgPool;

pub const CORE_SCOPE: &str = "core";

pub async fn get(pool: &PgPool, scope: &str) -> Result<Option<Value>, sqlx::Error> {
    sqlx::query_scalar("SELECT value FROM settings.entries WHERE scope = $1")
        .bind(scope)
        .fetch_optional(pool)
        .await
}

/// Every Module scope (everything except `core`), ordered by id.
pub async fn list_modules(pool: &PgPool) -> Result<Vec<(String, Value)>, sqlx::Error> {
    sqlx::query_as("SELECT scope, value FROM settings.entries WHERE scope <> $1 ORDER BY scope")
        .bind(CORE_SCOPE)
        .fetch_all(pool)
        .await
}

pub async fn upsert(pool: &PgPool, scope: &str, value: &Value) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO settings.entries (scope, value) VALUES ($1, $2)
         ON CONFLICT (scope) DO UPDATE SET value = EXCLUDED.value, updated_at = now()",
    )
    .bind(scope)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}
