//! The lock that lets only one confirm run at a time, so two confirms of overlapping
//! Statements cannot both see a row as new.

use sqlx::PgConnection;

/// Arbitrary but fixed (`fin_conf` as hex): the advisory lock every confirm takes.
const CONFIRM_LOCK_KEY: i64 = 0x0066_696e_5f63_6f6e;

/// Takes the confirm lock for the rest of the transaction; `false` when another confirm
/// holds it. Postgres releases it on commit or rollback, so a crash can never leave it stuck.
///
/// # Errors
/// The database failed.
pub async fn try_acquire(tx: &mut PgConnection) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT pg_try_advisory_xact_lock($1)")
        .bind(CONFIRM_LOCK_KEY)
        .fetch_one(tx)
        .await
}
