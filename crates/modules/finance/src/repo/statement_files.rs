//! Finding a Statement by the hash of its file, and moving it to a new file.

use sqlx::PgExecutor;

use super::statements::{COLUMNS, StatementRow};

/// The Statement whose kept original, or original upload, has this hash.
///
/// # Errors
/// The database failed.
pub async fn find_by_sha(
    db: impl PgExecutor<'_>,
    sha: &str,
) -> Result<Option<StatementRow>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM finance.statements
         WHERE file_sha256 = $1 OR upload_sha256 = $1 ORDER BY id LIMIT 1"
    )))
    .bind(sha)
    .fetch_optional(db)
    .await
}

/// Points a Statement at its decrypted original, remembering the encrypted upload's hash and
/// the label of the password that opened it.
///
/// # Errors
/// The database failed.
pub async fn record_unlock(
    db: impl PgExecutor<'_>,
    id: i64,
    decrypted_sha: &str,
    upload_sha: &str,
    label: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE finance.statements
         SET file_sha256 = $2, upload_sha256 = $3, unlocked_with_label = $4 WHERE id = $1",
    )
    .bind(id)
    .bind(decrypted_sha)
    .bind(upload_sha)
    .bind(label)
    .execute(db)
    .await?;
    Ok(())
}

/// The label of the password that unlocked the Statement, if any did.
///
/// # Errors
/// The database failed.
pub async fn unlocked_with_label(
    db: impl PgExecutor<'_>,
    id: i64,
) -> Result<Option<String>, sqlx::Error> {
    let label: Option<Option<String>> =
        sqlx::query_scalar("SELECT unlocked_with_label FROM finance.statements WHERE id = $1")
            .bind(id)
            .fetch_optional(db)
            .await?;
    Ok(label.flatten())
}

/// Whether any other Statement still points at this stored original.
///
/// # Errors
/// The database failed.
pub async fn file_in_use(db: impl PgExecutor<'_>, sha: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM finance.statements WHERE file_sha256 = $1)")
        .bind(sha)
        .fetch_one(db)
        .await
}
