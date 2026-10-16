//! `time.connection`: the single Toggl connection row.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

/// The stored connection. `Debug` is written by hand so the token can never reach a log.
#[derive(sqlx::FromRow)]
pub struct Connection {
    /// The person's Toggl API token. Write-only through the API.
    pub api_token: String,
    /// The workspace that is synced.
    pub workspace_id: i64,
    /// Its name, for display.
    pub workspace_name: String,
    /// Start time of the last successful Sync.
    pub sync_cursor: Option<DateTime<Utc>>,
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("workspace_id", &self.workspace_id)
            .field("api_token", &"<redacted>")
            .finish_non_exhaustive()
    }
}

/// The connection, or `None` when Toggl is not connected.
pub async fn get(pool: &PgPool) -> Result<Option<Connection>, sqlx::Error> {
    sqlx::query_as(
        "SELECT api_token, workspace_id, workspace_name, sync_cursor
         FROM time.connection",
    )
    .fetch_optional(pool)
    .await
}

/// Stores a new connection, replacing any old one. The Sync cursor starts over,
/// so the next Sync re-reads the whole window (safe: Sync is idempotent).
pub async fn save(
    pool: &PgPool,
    api_token: &str,
    workspace_id: i64,
    workspace_name: &str,
    toggl_user_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO time.connection (api_token, workspace_id, workspace_name, toggl_user_id)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (singleton) DO UPDATE SET
           api_token = EXCLUDED.api_token, workspace_id = EXCLUDED.workspace_id,
           workspace_name = EXCLUDED.workspace_name, toggl_user_id = EXCLUDED.toggl_user_id,
           connected_at = now(), sync_cursor = NULL",
    )
    .bind(api_token)
    .bind(workspace_id)
    .bind(workspace_name)
    .bind(toggl_user_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Removes the connection (and with it the token). Synced data stays.
pub async fn delete(pool: &PgPool) -> Result<bool, sqlx::Error> {
    let done = sqlx::query("DELETE FROM time.connection")
        .execute(pool)
        .await?;
    Ok(done.rows_affected() > 0)
}

/// Moves the Sync cursor; called inside the Sync's transaction.
pub async fn set_cursor(tx: &mut PgConnection, cursor: DateTime<Utc>) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE time.connection SET sync_cursor = $1")
        .bind(cursor)
        .execute(tx)
        .await?;
    Ok(())
}
