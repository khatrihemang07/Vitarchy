//! `finance.statement_passwords`: write-only. Only [`secrets`] ever returns a secret, and
//! only to be handed to the reader.

use std::fmt;

use chrono::{DateTime, Utc};
use sqlx::PgExecutor;

/// A saved password as the API may list it: no secret.
#[derive(Debug, sqlx::FromRow)]
pub struct PasswordLabel {
    /// Row id.
    pub id: i64,
    /// The name the person gave it.
    pub label: String,
    /// When it was saved.
    pub created_at: DateTime<Utc>,
    /// When it last opened a file.
    pub last_used_at: Option<DateTime<Utc>>,
}

/// A saved password with its secret. `Debug` hides the secret.
#[derive(sqlx::FromRow)]
pub struct StoredPassword {
    /// Row id.
    pub id: i64,
    /// The secret itself.
    pub secret: String,
}

impl fmt::Debug for StoredPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoredPassword")
            .field("id", &self.id)
            .field("secret", &"[hidden]")
            .finish()
    }
}

/// Saves a password and returns its id.
///
/// # Errors
/// The database failed.
pub async fn insert(
    db: impl PgExecutor<'_>,
    label: &str,
    secret: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO finance.statement_passwords (label, secret) VALUES ($1, $2) RETURNING id",
    )
    .bind(label)
    .bind(secret)
    .fetch_one(db)
    .await
}

/// The saved passwords without their secrets, oldest first.
///
/// # Errors
/// The database failed.
pub async fn labels(db: impl PgExecutor<'_>) -> Result<Vec<PasswordLabel>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, label, created_at, last_used_at FROM finance.statement_passwords ORDER BY id",
    )
    .fetch_all(db)
    .await
}

/// Every saved password with its secret, in the order they are tried: oldest first. The
/// position in this list is what the reader reports as `unlocked_with`.
///
/// # Errors
/// The database failed.
pub async fn secrets(db: impl PgExecutor<'_>) -> Result<Vec<StoredPassword>, sqlx::Error> {
    sqlx::query_as("SELECT id, secret FROM finance.statement_passwords ORDER BY id")
        .fetch_all(db)
        .await
}

/// Notes that a password just opened a file.
///
/// # Errors
/// The database failed.
pub async fn touch(db: impl PgExecutor<'_>, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE finance.statement_passwords SET last_used_at = now() WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

/// Forgets a password; `false` when it does not exist.
///
/// # Errors
/// The database failed.
pub async fn delete(db: impl PgExecutor<'_>, id: i64) -> Result<bool, sqlx::Error> {
    let done = sqlx::query("DELETE FROM finance.statement_passwords WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(done.rows_affected() == 1)
}
