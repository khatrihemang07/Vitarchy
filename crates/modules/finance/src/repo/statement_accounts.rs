//! `finance.statement_accounts`: the Accounts a Statement lists, in printed order.

use sqlx::PgExecutor;

/// One Account slot of a Statement.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct StatementAccountRow {
    /// Printed order, from 0.
    pub position: i32,
    /// The Account it resolved to, once known.
    pub account_id: Option<i64>,
    /// The issuer the reader found.
    pub proposed_issuer: Option<String>,
    /// The kind the reader found.
    pub proposed_kind: Option<String>,
    /// The mask the reader found.
    pub proposed_mask: Option<String>,
}

/// Adds one slot.
///
/// # Errors
/// The database failed.
pub async fn insert(
    db: impl PgExecutor<'_>,
    statement_id: i64,
    row: &StatementAccountRow,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO finance.statement_accounts
           (statement_id, position, account_id, proposed_issuer, proposed_kind, proposed_mask)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(statement_id)
    .bind(row.position)
    .bind(row.account_id)
    .bind(&row.proposed_issuer)
    .bind(&row.proposed_kind)
    .bind(&row.proposed_mask)
    .execute(db)
    .await?;
    Ok(())
}

/// A Statement's slots in printed order.
///
/// # Errors
/// The database failed.
pub async fn list(
    db: impl PgExecutor<'_>,
    statement_id: i64,
) -> Result<Vec<StatementAccountRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT position, account_id, proposed_issuer, proposed_kind, proposed_mask
         FROM finance.statement_accounts WHERE statement_id = $1 ORDER BY position",
    )
    .bind(statement_id)
    .fetch_all(db)
    .await
}

/// Points one slot at an Account (a match, or the person's reassignment).
///
/// # Errors
/// The database failed.
pub async fn assign(
    db: impl PgExecutor<'_>,
    statement_id: i64,
    position: i32,
    account_id: i64,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE finance.statement_accounts SET account_id = $3 WHERE statement_id = $1 AND position = $2",
    )
    .bind(statement_id)
    .bind(position)
    .bind(account_id)
    .execute(db)
    .await?;
    Ok(done.rows_affected() == 1)
}

/// Removes all of a Statement's slots (before they are proposed afresh by a Read again).
///
/// # Errors
/// The database failed.
pub async fn clear(db: impl PgExecutor<'_>, statement_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM finance.statement_accounts WHERE statement_id = $1")
        .bind(statement_id)
        .execute(db)
        .await?;
    Ok(())
}
