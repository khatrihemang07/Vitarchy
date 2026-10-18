//! `time.timeline_meta`: which Away rules the stored Timeline was built under.

use sqlx::{PgConnection, PgExecutor};

/// The rules fingerprint the Timeline was last rebuilt under; `None` before any rebuild recorded one.
pub async fn rules_fingerprint<'e>(
    executor: impl PgExecutor<'e>,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT rules_fingerprint FROM time.timeline_meta")
        .fetch_optional(executor)
        .await
}

/// Records `fingerprint` as the rules the Timeline was built under.
pub async fn set_rules_fingerprint(
    tx: &mut PgConnection,
    fingerprint: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO time.timeline_meta (only_row, rules_fingerprint) VALUES (true, $1)
         ON CONFLICT (only_row) DO UPDATE SET rules_fingerprint = EXCLUDED.rules_fingerprint",
    )
    .bind(fingerprint)
    .execute(&mut *tx)
    .await?;
    Ok(())
}
