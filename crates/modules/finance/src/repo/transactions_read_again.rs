//! The `finance.transactions` queries behind Read again: what is stored, and the marks.

use sqlx::PgExecutor;

use crate::{
    dedupe::{Evidence, StoredEvidence},
    reader::contract::Direction,
};

#[derive(sqlx::FromRow)]
struct KeyRow {
    id: i64,
    date: chrono::NaiveDate,
    amount_paise: i64,
    direction: String,
    source_id: Option<String>,
    balance_paise: Option<i64>,
    reference: Option<String>,
    description: String,
    lost: bool,
}

/// The Account's stored rows as the Read again diff sees them, in id order. All of the
/// Account's rows are compared, not only one Statement's, because overlapping Statements
/// share rows.
///
/// # Errors
/// The database failed.
pub async fn stored_keys(
    db: impl PgExecutor<'_>,
    account_id: i64,
) -> Result<Vec<StoredEvidence>, sqlx::Error> {
    let rows: Vec<KeyRow> = sqlx::query_as(
        "SELECT id, date, amount_paise, direction, source_id, balance_paise, reference, description,
                no_longer_found_at IS NOT NULL AS lost
         FROM finance.transactions WHERE account_id = $1 ORDER BY id",
    )
    .bind(account_id)
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            Some(StoredEvidence {
                id: r.id,
                evidence: Evidence::new(
                    r.date,
                    r.amount_paise,
                    Direction::from_id(&r.direction)?,
                    r.source_id.as_deref(),
                    r.balance_paise,
                    r.reference.as_deref(),
                    &r.description,
                ),
                lost: r.lost,
            })
        })
        .collect())
}

/// Marks rows a Read again no longer finds. Never deletes.
///
/// # Errors
/// The database failed.
pub async fn mark_no_longer_found(
    db: impl PgExecutor<'_>,
    ids: &[i64],
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE finance.transactions SET no_longer_found_at = now()
         WHERE id = ANY($1) AND no_longer_found_at IS NULL",
    )
    .bind(ids)
    .execute(db)
    .await?;
    Ok(done.rows_affected())
}

/// Clears the mark on rows a Read again finds once more; returns how many had one.
///
/// # Errors
/// The database failed.
pub async fn clear_no_longer_found(
    db: impl PgExecutor<'_>,
    ids: &[i64],
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        "UPDATE finance.transactions SET no_longer_found_at = NULL
         WHERE id = ANY($1) AND no_longer_found_at IS NOT NULL",
    )
    .bind(ids)
    .execute(db)
    .await?;
    Ok(done.rows_affected())
}

/// The ids of the rows a Statement contributed to an Account.
///
/// # Errors
/// The database failed.
pub async fn ids_of_statement(
    db: impl PgExecutor<'_>,
    account_id: i64,
    statement_id: i64,
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT id FROM finance.transactions WHERE account_id = $1 AND statement_id = $2 ORDER BY id",
    )
    .bind(account_id)
    .bind(statement_id)
    .fetch_all(db)
    .await
}
