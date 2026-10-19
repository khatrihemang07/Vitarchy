//! `finance.markers`: one-time jobs that have already run.

use sqlx::PgExecutor;

/// Whether the job `name` has run.
///
/// # Errors
/// The database failed.
pub async fn is_done(db: impl PgExecutor<'_>, name: &str) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM finance.markers WHERE name = $1)")
        .bind(name)
        .fetch_one(db)
        .await
}

/// Records that the job `name` has run; repeats change nothing.
///
/// # Errors
/// The database failed.
pub async fn mark_done(db: impl PgExecutor<'_>, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO finance.markers (name) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(name)
        .execute(db)
        .await?;
    Ok(())
}
