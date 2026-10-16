//! `time.connection`: the single Toggl connection row.

use chrono::{DateTime, Utc};
use sqlx::{PgConnection, PgPool};

use super::{catalog, entries, mirror_owner};
use crate::sync::account::{Account, mirror_is_foreign};

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
///
/// When `account` is not the one the mirror was made for, the mirrored Time Entries,
/// projects, clients and tags are cleared in the same transaction (Toggl owns them, so
/// the next Sync fetches them again). Activity is never touched. Returns whether the
/// mirror was cleared.
///
/// The connection row is written first: that waits for a running Sync to finish
/// (see [`confirm_current`]), so the clear can never race its writes.
pub async fn save(
    pool: &PgPool,
    api_token: &str,
    workspace_name: &str,
    account: Account,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let owner = mirror_owner::get_for_update(&mut tx).await?;
    sqlx::query(
        "INSERT INTO time.connection (api_token, workspace_id, workspace_name, toggl_user_id)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (singleton) DO UPDATE SET
           api_token = EXCLUDED.api_token, workspace_id = EXCLUDED.workspace_id,
           workspace_name = EXCLUDED.workspace_name, toggl_user_id = EXCLUDED.toggl_user_id,
           connected_at = now(), sync_cursor = NULL",
    )
    .bind(api_token)
    .bind(account.workspace_id)
    .bind(workspace_name)
    .bind(account.user_id)
    .execute(&mut *tx)
    .await?;
    let foreign = mirror_is_foreign(owner, account);
    if foreign {
        entries::clear(&mut tx).await?;
        catalog::clear(&mut tx).await?;
    }
    mirror_owner::set(&mut tx, account).await?;
    tx.commit().await?;
    Ok(foreign)
}

/// Removes the connection (and with it the token). Synced data stays.
pub async fn delete(pool: &PgPool) -> Result<bool, sqlx::Error> {
    let done = sqlx::query("DELETE FROM time.connection")
        .execute(pool)
        .await?;
    Ok(done.rows_affected() > 0)
}

/// Share-locks the connection row if it still has this token and workspace, so a
/// reconnect cannot change it until the transaction ends. `false` when it already
/// changed (or went), meaning the Sync that read it must not write anything.
pub async fn confirm_current(
    tx: &mut PgConnection,
    api_token: &str,
    workspace_id: i64,
) -> Result<bool, sqlx::Error> {
    let row: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM time.connection WHERE api_token = $1 AND workspace_id = $2 FOR SHARE",
    )
    .bind(api_token)
    .bind(workspace_id)
    .fetch_optional(tx)
    .await?;
    Ok(row.is_some())
}

/// Moves the Sync cursor, but only on the connection (token and workspace) the Sync
/// started with; called inside the Sync's transaction. `false` when it no longer matches.
pub async fn set_cursor(
    tx: &mut PgConnection,
    cursor: DateTime<Utc>,
    api_token: &str,
    workspace_id: i64,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE time.connection SET sync_cursor = $1 WHERE api_token = $2 AND workspace_id = $3",
    )
    .bind(cursor)
    .bind(api_token)
    .bind(workspace_id)
    .execute(tx)
    .await?;
    Ok(done.rows_affected() > 0)
}
