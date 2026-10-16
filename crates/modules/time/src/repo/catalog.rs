//! `time.projects`, `time.clients` and `time.tags`: the workspace's lookup lists.

use sqlx::PgConnection;

use crate::toggl::types::{Raw, TogglClient, TogglProject, TogglTag};

/// Inserts or updates projects.
pub async fn upsert_projects(
    tx: &mut PgConnection,
    workspace_id: i64,
    projects: &[Raw<TogglProject>],
) -> Result<(), sqlx::Error> {
    for Raw { item, raw } in projects {
        sqlx::query(
            "INSERT INTO time.projects (toggl_id, workspace_id, client_id, name, active, color, raw)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (toggl_id) DO UPDATE SET
               client_id = EXCLUDED.client_id, name = EXCLUDED.name, active = EXCLUDED.active,
               color = EXCLUDED.color, raw = EXCLUDED.raw, synced_at = now()",
        )
        .bind(item.id)
        .bind(workspace_id)
        .bind(item.client_id)
        .bind(&item.name)
        .bind(item.active.unwrap_or(true))
        .bind(&item.color)
        .bind(raw)
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}

/// Inserts or updates clients.
pub async fn upsert_clients(
    tx: &mut PgConnection,
    workspace_id: i64,
    clients: &[Raw<TogglClient>],
) -> Result<(), sqlx::Error> {
    for Raw { item, raw } in clients {
        sqlx::query(
            "INSERT INTO time.clients (toggl_id, workspace_id, name, archived, raw)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (toggl_id) DO UPDATE SET
               name = EXCLUDED.name, archived = EXCLUDED.archived, raw = EXCLUDED.raw,
               synced_at = now()",
        )
        .bind(item.id)
        .bind(workspace_id)
        .bind(&item.name)
        .bind(item.archived.unwrap_or(false))
        .bind(raw)
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}

/// Inserts or updates tags.
pub async fn upsert_tags(
    tx: &mut PgConnection,
    workspace_id: i64,
    tags: &[Raw<TogglTag>],
) -> Result<(), sqlx::Error> {
    for Raw { item, raw } in tags {
        sqlx::query(
            "INSERT INTO time.tags (toggl_id, workspace_id, name, raw) VALUES ($1, $2, $3, $4)
             ON CONFLICT (toggl_id) DO UPDATE SET
               name = EXCLUDED.name, raw = EXCLUDED.raw, synced_at = now()",
        )
        .bind(item.id)
        .bind(workspace_id)
        .bind(&item.name)
        .bind(raw)
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}
