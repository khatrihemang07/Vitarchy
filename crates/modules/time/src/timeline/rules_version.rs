//! Rebuilding the Timeline once when the Away rules change, so stored Away matches the code.

use sqlx::PgPool;

use super::{TimeRange, rebuild::rebuild_within, rules_fingerprint};
use crate::{
    error::TimeError,
    repo::{timeline, timeline_meta},
};

/// Brings the stored Timeline up to the current rules. The usual case is one cheap read. When
/// the stored fingerprint differs, rebuilds the whole Timeline and records the fingerprint in
/// one transaction under the rebuild lock, so concurrent callers rebuild once. Returns whether
/// it rebuilt. The `Module` contract has no startup hook, so readers of the Timeline call this
/// first.
///
/// # Errors
/// A failed database read or write; nothing is changed then.
pub async fn ensure_rules_current(pool: &PgPool) -> Result<bool, TimeError> {
    let current = rules_fingerprint::current();
    if timeline_meta::rules_fingerprint(pool).await?.as_deref() == Some(current.as_str()) {
        return Ok(false);
    }
    let mut tx = pool.begin().await?;
    timeline::lock_rebuilds(&mut tx).await?;
    // Another caller may have rebuilt while this one waited for the lock.
    let stored = timeline_meta::rules_fingerprint(&mut *tx).await?;
    if stored.as_deref() == Some(current.as_str()) {
        return Ok(false);
    }
    let rebuilt = rebuild_within(&mut tx, TimeRange::everything()).await?;
    timeline_meta::set_rules_fingerprint(&mut tx, &current).await?;
    tx.commit().await?;
    tracing::info!(
        had_fingerprint = stored.is_some(),
        activities = rebuilt.activities,
        segments = rebuilt.segments,
        "timeline rebuilt for new Away rules"
    );
    Ok(true)
}

/// [`ensure_rules_current`] for readers: a failed rebuild is logged and the existing Timeline
/// is served, so a status or a page never breaks because of it. The next read tries again.
pub async fn ensure_rules_current_or_warn(pool: &PgPool) {
    if let Err(error) = ensure_rules_current(pool).await {
        tracing::warn!(
            error_code = error.code(),
            "could not rebuild the timeline for new Away rules; serving the stored one"
        );
    }
}
