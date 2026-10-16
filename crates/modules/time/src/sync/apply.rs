//! Writes one Sync's results in a single transaction, moving the cursor last,
//! so a failure leaves both the data and the cursor as they were.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::{
    counts::{SyncCounts, count},
    fetch::Fetched,
    plan::Plan,
};
use crate::repo::{catalog, connection, entries};

/// Stores `plan` and the catalog lists, then sets the cursor to `started_at`.
pub async fn apply(
    pool: &PgPool,
    workspace_id: i64,
    fetched: &Fetched,
    plan: &Plan,
    started_at: DateTime<Utc>,
) -> Result<SyncCounts, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let mut counts = SyncCounts {
        skipped_running: plan.skipped_running,
        projects: count(fetched.projects.len()),
        clients: count(fetched.clients.len()),
        tags: count(fetched.tags.len()),
        ..SyncCounts::default()
    };
    for row in &plan.upserts {
        if entries::upsert(&mut tx, row).await? {
            counts.inserted += 1;
        } else {
            counts.updated += 1;
        }
    }
    counts.deleted =
        u32::try_from(entries::delete(&mut tx, &plan.deletes).await?).unwrap_or(u32::MAX);
    catalog::upsert_projects(&mut tx, workspace_id, &fetched.projects).await?;
    catalog::upsert_clients(&mut tx, workspace_id, &fetched.clients).await?;
    catalog::upsert_tags(&mut tx, workspace_id, &fetched.tags).await?;
    connection::set_cursor(&mut tx, started_at).await?;
    tx.commit().await?;
    Ok(counts)
}
