//! `finance.printings`: every Statement's Printing of a Transaction (ADR 0015).

use std::collections::HashMap;

use sqlx::PgExecutor;

use crate::reader::contract::Transaction;

pub use super::printings_load::{for_account, for_transactions};

/// Stores `statement_id`'s Printing of a Transaction. `false` when that Statement already has
/// one for it (nothing is written then).
///
/// # Errors
/// The database failed.
pub async fn insert(
    db: impl PgExecutor<'_>,
    transaction_id: i64,
    statement_id: i64,
    tx: &Transaction,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "INSERT INTO finance.printings
           (transaction_id, statement_id, date, description, balance_paise, reference, source_id,
            foreign_amount, foreign_currency)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT (statement_id, transaction_id) DO NOTHING",
    )
    .bind(transaction_id)
    .bind(statement_id)
    .bind(tx.date)
    .bind(&tx.description)
    .bind(tx.balance_paise)
    .bind(&tx.reference)
    .bind(&tx.source_id)
    .bind(&tx.foreign_amount)
    .bind(&tx.foreign_currency)
    .execute(db)
    .await?;
    Ok(done.rows_affected() == 1)
}

/// Marks (`gone`) or clears the Statement's Printings of these Transactions. Returns the
/// Transactions whose Printing actually changed. Never deletes.
///
/// # Errors
/// The database failed.
pub async fn set_gone(
    db: impl PgExecutor<'_>,
    statement_id: i64,
    transaction_ids: &[i64],
    gone: bool,
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "UPDATE finance.printings
         SET gone_at = CASE WHEN $3 THEN now() ELSE NULL END
         WHERE statement_id = $1 AND transaction_id = ANY($2) AND (gone_at IS NOT NULL) <> $3
         RETURNING transaction_id",
    )
    .bind(statement_id)
    .bind(transaction_ids)
    .bind(gone)
    .fetch_all(db)
    .await
}

/// The Transactions of an Account that a Statement printed, each with whether that Printing
/// is gone.
///
/// # Errors
/// The database failed.
pub async fn marks_of_statement(
    db: impl PgExecutor<'_>,
    account_id: i64,
    statement_id: i64,
) -> Result<HashMap<i64, bool>, sqlx::Error> {
    let rows: Vec<(i64, bool)> = sqlx::query_as(
        "SELECT p.transaction_id, p.gone_at IS NOT NULL
         FROM finance.printings p JOIN finance.transactions t ON t.id = p.transaction_id
         WHERE p.statement_id = $1 AND t.account_id = $2",
    )
    .bind(statement_id)
    .bind(account_id)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().collect())
}
