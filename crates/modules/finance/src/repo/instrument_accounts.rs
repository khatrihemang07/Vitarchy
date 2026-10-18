//! `finance.instrument_accounts`: which Account each payment-app instrument is, remembered
//! from the person's confirms.

use std::collections::HashMap;

use sqlx::PgExecutor;

/// Every remembered instrument and its Account, by [`crate::accounts::instrument_key`].
///
/// # Errors
/// The database failed.
pub async fn all(db: impl PgExecutor<'_>) -> Result<HashMap<String, i64>, sqlx::Error> {
    let rows: Vec<(String, i64)> =
        sqlx::query_as("SELECT instrument, account_id FROM finance.instrument_accounts")
            .fetch_all(db)
            .await?;
    Ok(rows.into_iter().collect())
}

/// Remembers that `instrument` (a [`crate::accounts::instrument_key`]) is `account_id`; the
/// latest choice replaces an earlier one.
///
/// # Errors
/// The database failed.
pub async fn remember(
    db: impl PgExecutor<'_>,
    instrument: &str,
    account_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO finance.instrument_accounts (instrument, account_id) VALUES ($1, $2)
         ON CONFLICT (instrument) DO UPDATE SET account_id = EXCLUDED.account_id",
    )
    .bind(instrument)
    .bind(account_id)
    .execute(db)
    .await?;
    Ok(())
}
