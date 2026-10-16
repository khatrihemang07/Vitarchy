//! The lock that lets only one Sync run at a time.

use sqlx::PgConnection;

/// Arbitrary but fixed (`time_syn` as hex): the advisory lock every Sync takes.
const SYNC_LOCK_KEY: i64 = 0x7469_6d65_5f73_796e;

/// Takes the Sync lock for the rest of the transaction; `false` when another Sync holds it.
/// Postgres releases it on commit or rollback, so a crashed Sync can never leave it stuck.
pub async fn try_acquire(tx: &mut PgConnection) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
        .bind(SYNC_LOCK_KEY)
        .fetch_one(tx)
        .await
}
