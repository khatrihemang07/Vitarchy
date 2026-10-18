//! Changes to a Statement's state: confirming, replacing its result, discarding.

use sqlx::PgExecutor;

use super::statements::NewStatement;

/// Marks an unconfirmed Statement confirmed; `false` when it is missing or already confirmed.
///
/// # Errors
/// The database failed.
pub async fn confirm(db: impl PgExecutor<'_>, id: i64) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE finance.statements SET status = 'confirmed', confirmed_at = now()
         WHERE id = $1 AND status = 'unconfirmed'",
    )
    .bind(id)
    .execute(db)
    .await?;
    Ok(done.rows_affected() == 1)
}

/// Replaces the stored result after a Read again.
///
/// # Errors
/// The database failed.
pub async fn replace_result(
    db: impl PgExecutor<'_>,
    id: i64,
    new: &NewStatement<'_>,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE finance.statements
         SET format = $2, kind = $3, outcome = $4, result = $5, reader_version = $6, read_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(new.format)
    .bind(new.kind)
    .bind(new.outcome)
    .bind(new.result)
    .bind(new.reader_version)
    .execute(db)
    .await?;
    Ok(done.rows_affected() == 1)
}

/// Discards an unconfirmed Statement; `false` when it is missing or confirmed.
///
/// # Errors
/// The database failed.
pub async fn discard(db: impl PgExecutor<'_>, id: i64) -> Result<bool, sqlx::Error> {
    let done =
        sqlx::query("DELETE FROM finance.statements WHERE id = $1 AND status = 'unconfirmed'")
            .bind(id)
            .execute(db)
            .await?;
    Ok(done.rows_affected() == 1)
}
