//! The writes shared by confirm and Read again (ADR 0015): new Transactions with their first
//! Printing, and Printings added to Transactions other Statements stored.

use sqlx::PgConnection;

use super::refresh::refresh;
use crate::{
    printings::{Plan, plan_for},
    reader::contract::Transaction,
    repo::{printings, transactions},
};

/// A row of a Statement and the stored Transaction it is the same as.
#[derive(Clone, Copy)]
pub struct Target<'a> {
    /// The stored Transaction that gains a Printing.
    pub transaction_id: i64,
    /// The row the Statement printed.
    pub row: &'a Transaction,
}

/// What adding Printings did.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PrintingsAdded {
    /// Printings stored (a row already printed, or printed exactly like a kept one, adds none).
    pub added: usize,
    /// Of those, ones that made the Transaction show more detail.
    pub enriched: usize,
}

/// For each target, what Statement `statement_id` printing it would do. Preview, Read again
/// and confirm all count through this, so their numbers agree.
///
/// # Errors
/// The database failed.
pub async fn plan_by_target(
    db: &mut PgConnection,
    statement_id: i64,
    targets: &[Target<'_>],
) -> Result<Vec<Plan>, sqlx::Error> {
    let ids: Vec<i64> = targets.iter().map(|t| t.transaction_id).collect();
    let existing = printings::for_transactions(&mut *db, &ids).await?;
    Ok(targets
        .iter()
        .map(|t| {
            let list = existing.get(&t.transaction_id).map(Vec::as_slice);
            plan_for(statement_id, t.row, list)
        })
        .collect())
}

/// Stores `rows` as new Transactions of the Account, each with its first Printing.
///
/// # Errors
/// The database failed.
pub async fn add_transactions(
    db: &mut PgConnection,
    account_id: i64,
    statement_id: i64,
    rows: &[&Transaction],
) -> Result<usize, sqlx::Error> {
    for row in rows {
        let id = transactions::insert(&mut *db, account_id, statement_id, row).await?;
        printings::insert(&mut *db, id, statement_id, row).await?;
    }
    Ok(rows.len())
}

/// Stores Statement `statement_id`'s Printing for each target and re-resolves the
/// Transactions that gained one.
///
/// # Errors
/// The database failed.
pub async fn add_printings(
    db: &mut PgConnection,
    statement_id: i64,
    targets: &[Target<'_>],
) -> Result<PrintingsAdded, sqlx::Error> {
    let plans = plan_by_target(db, statement_id, targets).await?;
    let skipped_identical = plans.iter().filter(|p| **p == Plan::IdenticalKept).count();
    tracing::debug!(
        statement_id,
        rows = targets.len(),
        skipped_identical,
        "printings planned"
    );
    let mut done = PrintingsAdded::default();
    let mut touched = Vec::new();
    for (target, plan) in targets.iter().zip(plans) {
        let Plan::Add { gains } = plan else {
            continue;
        };
        if printings::insert(&mut *db, target.transaction_id, statement_id, target.row).await? {
            done.added += 1;
            done.enriched += usize::from(gains);
            touched.push(target.transaction_id);
        }
    }
    refresh(db, &touched).await?;
    Ok(done)
}
