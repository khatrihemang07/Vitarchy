//! Writes one Sync's results inside the Sync's transaction, moving the cursor last,
//! so a failure leaves both the data and the cursor as they were.

use chrono::{DateTime, Utc};
use sqlx::PgConnection;

use super::{counts::SyncCounts, fetch::Fetched, plan::Plan};
use crate::{
    error::TimeError,
    repo::{catalog, connection, connection::Connection, entries},
    saturating::saturating_u32,
};

/// Stores `plan` and the catalog lists, then sets the cursor to `started_at`.
/// The caller commits; any error means nothing was kept.
///
/// # Errors
/// [`TimeError::ConnectionChanged`] when the connection is no longer the one the
/// Sync started with; a database error otherwise.
pub async fn apply(
    tx: &mut PgConnection,
    connection: &Connection,
    fetched: &Fetched,
    plan: &Plan,
    started_at: DateTime<Utc>,
) -> Result<SyncCounts, TimeError> {
    let (token, workspace_id) = (connection.api_token.as_str(), connection.workspace_id);
    // Also blocks a reconnect until this transaction ends.
    if !connection::confirm_current(tx, token, workspace_id).await? {
        return Err(TimeError::ConnectionChanged);
    }
    let written = entries::upsert_all(tx, &plan.upserts).await?;
    let mut deleted = entries::delete(tx, &plan.deletes).await?;
    if let Some(span) = &plan.reconcile {
        deleted += entries::delete_unlisted(tx, workspace_id, span).await?;
    }
    catalog::upsert_projects(tx, workspace_id, &fetched.projects).await?;
    catalog::upsert_clients(tx, workspace_id, &fetched.clients).await?;
    catalog::upsert_tags(tx, workspace_id, &fetched.tags).await?;
    if !connection::set_cursor(tx, started_at, token, workspace_id).await? {
        return Err(TimeError::ConnectionChanged);
    }
    Ok(SyncCounts {
        inserted: saturating_u32(written.inserted),
        updated: saturating_u32(written.updated),
        deleted: saturating_u32(deleted),
        skipped_running: plan.skipped_running,
        skipped_other_workspace: plan.skipped_other_workspace,
        projects: saturating_u32(fetched.projects.len()),
        clients: saturating_u32(fetched.clients.len()),
        tags: saturating_u32(fetched.tags.len()),
    })
}
