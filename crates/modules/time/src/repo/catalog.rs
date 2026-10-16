//! `time.projects`, `time.clients` and `time.tags`: the workspace's lookup lists.

use std::collections::BTreeMap;

use sqlx::PgConnection;

use crate::toggl::types::{Raw, TogglClient, TogglProject, TogglTag};

/// One record per Toggl id, the last listed winning: a list read in pages can repeat
/// a record when Toggl changes it mid-read, and one statement cannot touch a row twice.
fn distinct<T>(records: &[Raw<T>], id: impl Fn(&T) -> i64) -> Vec<&Raw<T>> {
    let by_id: BTreeMap<i64, &Raw<T>> = records.iter().map(|r| (id(&r.item), r)).collect();
    by_id.into_values().collect()
}

fn raw_texts<T>(records: &[&Raw<T>]) -> Vec<String> {
    records.iter().map(|r| r.raw.to_string()).collect()
}

/// Inserts or updates projects.
pub async fn upsert_projects(
    tx: &mut PgConnection,
    workspace_id: i64,
    projects: &[Raw<TogglProject>],
) -> Result<(), sqlx::Error> {
    let rows = distinct(projects, |p| p.id);
    let ids: Vec<i64> = rows.iter().map(|r| r.item.id).collect();
    let clients: Vec<Option<i64>> = rows.iter().map(|r| r.item.client_id).collect();
    let names: Vec<&str> = rows.iter().map(|r| r.item.name.as_str()).collect();
    let active: Vec<bool> = rows.iter().map(|r| r.item.active.unwrap_or(true)).collect();
    let colors: Vec<Option<&str>> = rows.iter().map(|r| r.item.color.as_deref()).collect();
    sqlx::query(
        "INSERT INTO time.projects (toggl_id, workspace_id, client_id, name, active, color, raw)
         SELECT i.id, $1, i.client_id, i.name, i.active, i.color, i.raw::jsonb
         FROM UNNEST($2::bigint[], $3::bigint[], $4::text[], $5::bool[], $6::text[], $7::text[])
           AS i(id, client_id, name, active, color, raw)
         ON CONFLICT (toggl_id) DO UPDATE SET
           client_id = EXCLUDED.client_id, name = EXCLUDED.name, active = EXCLUDED.active,
           color = EXCLUDED.color, raw = EXCLUDED.raw, synced_at = now()",
    )
    .bind(workspace_id)
    .bind(&ids)
    .bind(&clients)
    .bind(&names)
    .bind(&active)
    .bind(&colors)
    .bind(raw_texts(&rows))
    .execute(tx)
    .await?;
    Ok(())
}

/// Inserts or updates clients.
pub async fn upsert_clients(
    tx: &mut PgConnection,
    workspace_id: i64,
    clients: &[Raw<TogglClient>],
) -> Result<(), sqlx::Error> {
    let rows = distinct(clients, |c| c.id);
    let ids: Vec<i64> = rows.iter().map(|r| r.item.id).collect();
    let names: Vec<&str> = rows.iter().map(|r| r.item.name.as_str()).collect();
    let archived: Vec<bool> = rows
        .iter()
        .map(|r| r.item.archived.unwrap_or(false))
        .collect();
    sqlx::query(
        "INSERT INTO time.clients (toggl_id, workspace_id, name, archived, raw)
         SELECT i.id, $1, i.name, i.archived, i.raw::jsonb
         FROM UNNEST($2::bigint[], $3::text[], $4::bool[], $5::text[])
           AS i(id, name, archived, raw)
         ON CONFLICT (toggl_id) DO UPDATE SET
           name = EXCLUDED.name, archived = EXCLUDED.archived, raw = EXCLUDED.raw,
           synced_at = now()",
    )
    .bind(workspace_id)
    .bind(&ids)
    .bind(&names)
    .bind(&archived)
    .bind(raw_texts(&rows))
    .execute(tx)
    .await?;
    Ok(())
}

/// Inserts or updates tags.
pub async fn upsert_tags(
    tx: &mut PgConnection,
    workspace_id: i64,
    tags: &[Raw<TogglTag>],
) -> Result<(), sqlx::Error> {
    let rows = distinct(tags, |t| t.id);
    let ids: Vec<i64> = rows.iter().map(|r| r.item.id).collect();
    let names: Vec<&str> = rows.iter().map(|r| r.item.name.as_str()).collect();
    sqlx::query(
        "INSERT INTO time.tags (toggl_id, workspace_id, name, raw)
         SELECT i.id, $1, i.name, i.raw::jsonb
         FROM UNNEST($2::bigint[], $3::text[], $4::text[]) AS i(id, name, raw)
         ON CONFLICT (toggl_id) DO UPDATE SET
           name = EXCLUDED.name, raw = EXCLUDED.raw, synced_at = now()",
    )
    .bind(workspace_id)
    .bind(&ids)
    .bind(&names)
    .bind(raw_texts(&rows))
    .execute(tx)
    .await?;
    Ok(())
}

/// Empties the projects, clients and tags (Toggl owns them, so they can be fetched again).
pub async fn clear(tx: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query("TRUNCATE time.projects, time.clients, time.tags")
        .execute(tx)
        .await?;
    Ok(())
}
