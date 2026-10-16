//! Saving a scope: store the document and log which keys changed.

use serde_json::Value;
use sqlx::PgPool;

use crate::{changes::changed_keys, repo};

/// Replaces a scope's document and logs `scope` plus the *names* of the keys
/// that changed. Values are never logged: they are the user's data.
///
/// # Errors
/// The database failure, if the write did not happen.
pub async fn save_scope(pool: &PgPool, scope: &str, new: &Value) -> Result<(), sqlx::Error> {
    let previous = repo::upsert(pool, scope, new).await?;
    tracing::info!(
        scope,
        changed = ?changed_keys(previous.as_ref(), new),
        "settings saved"
    );
    Ok(())
}
