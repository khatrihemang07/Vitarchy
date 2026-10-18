//! Row counts for the status page.

use sqlx::PgExecutor;

/// Accounts stored.
///
/// # Errors
/// The database failed.
pub async fn accounts(db: impl PgExecutor<'_>) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM finance.accounts")
        .fetch_one(db)
        .await
}

/// Statements waiting for the person to confirm them.
///
/// # Errors
/// The database failed.
pub async fn unconfirmed_statements(db: impl PgExecutor<'_>) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM finance.statements WHERE status = 'unconfirmed'")
        .fetch_one(db)
        .await
}

/// Payment records not yet linked to a Transaction.
///
/// # Errors
/// The database failed.
pub async fn unlinked_payment_records(db: impl PgExecutor<'_>) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM finance.payment_records WHERE transaction_id IS NULL")
        .fetch_one(db)
        .await
}
