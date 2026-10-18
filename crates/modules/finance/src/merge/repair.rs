//! Merging every group of duplicate Accounts, in one transaction under the confirm lock.

use std::time::Instant;

use sqlx::PgPool;

use super::{apply::merge_into, groups::duplicate_groups};
use crate::{
    error::FinanceError,
    import::link_waiting,
    repo::{accounts, confirm_lock},
};

/// What a repair did, as counts only.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MergeReport {
    /// Groups of duplicate Accounts found.
    pub groups: usize,
    /// Accounts merged away (deleted).
    pub accounts_merged: usize,
    /// Transactions moved to the kept Account as they were.
    pub moved: usize,
    /// Transactions folded into a kept one.
    pub folded: usize,
    /// Printings that moved onto kept Transactions.
    pub printings: u64,
    /// Payment records linked again afterwards.
    pub relinked: usize,
}

/// Merges Accounts with the same issuer, kind and visible digits into the oldest of each
/// group. Safe to run again: with no duplicates left it changes nothing.
///
/// # Errors
/// A confirm is running, or the database failed. Nothing is written then.
pub async fn merge_duplicate_accounts(pool: &PgPool) -> Result<MergeReport, FinanceError> {
    let timer = Instant::now();
    let mut tx = pool.begin().await?;
    if !confirm_lock::try_acquire(&mut tx).await? {
        return Err(FinanceError::ConfirmInProgress);
    }
    let groups = duplicate_groups(&accounts::own_identities(&mut *tx).await?);
    let mut report = MergeReport {
        groups: groups.len(),
        ..MergeReport::default()
    };
    for group in &groups {
        for from_id in &group.from {
            let merged = merge_into(&mut tx, group.keep, *from_id).await?;
            tracing::info!(
                keep_id = group.keep,
                from_id,
                moved = merged.moved,
                folded = merged.folded,
                "account merged"
            );
            report.accounts_merged += 1;
            report.moved += merged.moved;
            report.folded += merged.folded;
            report.printings += merged.printings;
        }
    }
    report.relinked = link_waiting::link_waiting(&mut tx).await?;
    tx.commit().await?;
    tracing::info!(
        groups = report.groups,
        accounts_merged = report.accounts_merged,
        moved = report.moved,
        folded = report.folded,
        relinked = report.relinked,
        duration_ms = u64::try_from(timer.elapsed().as_millis()).unwrap_or(u64::MAX),
        "duplicate accounts merged"
    );
    Ok(report)
}
