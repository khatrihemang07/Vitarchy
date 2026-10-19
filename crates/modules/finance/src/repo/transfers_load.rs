//! Reading Transfers back: for detection (all of them) and for display (a few by id).

use std::collections::HashSet;

use sqlx::PgExecutor;

use crate::transfers::{ExistingTransfer, Parts, Rung};

#[derive(sqlx::FromRow)]
struct ExistingRow {
    debit_id: i64,
    #[sqlx(try_from = "String")]
    rung: Rung,
    credit_ids: Vec<i64>,
    linked_paise: i64,
}

/// Every stored Transfer with its credits and what they add up to.
///
/// # Errors
/// The database failed.
pub async fn all(db: impl PgExecutor<'_>) -> Result<Vec<ExistingTransfer>, sqlx::Error> {
    let rows: Vec<ExistingRow> = sqlx::query_as(
        "SELECT t.debit_id, t.rung,
                array_agg(c.credit_id ORDER BY c.credit_id) AS credit_ids,
                sum(x.amount_paise)::bigint AS linked_paise
         FROM finance.transfers t
         JOIN finance.transfer_credits c ON c.transfer_id = t.id
         JOIN finance.transactions x ON x.id = c.credit_id
         GROUP BY t.id ORDER BY t.id",
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ExistingTransfer {
            debit_id: r.debit_id,
            credit_ids: r.credit_ids,
            linked_paise: r.linked_paise,
            rung: r.rung,
        })
        .collect())
}

/// Every (debit, credit) pair the person unlinked.
///
/// # Errors
/// The database failed.
pub async fn rejected_pairs(db: impl PgExecutor<'_>) -> Result<HashSet<(i64, i64)>, sqlx::Error> {
    let rows: Vec<(i64, i64)> =
        sqlx::query_as("SELECT debit_id, credit_id FROM finance.transfer_rejections")
            .fetch_all(db)
            .await?;
    Ok(rows.into_iter().collect())
}

#[derive(sqlx::FromRow)]
struct PartsRow {
    id: i64,
    #[sqlx(try_from = "String")]
    rung: Rung,
    debit_account_id: i64,
    debit_paise: i64,
    credit_account_ids: Vec<i64>,
    credited_paise: i64,
}

const PARTS: &str = "SELECT t.id, t.rung, d.account_id AS debit_account_id, d.amount_paise AS debit_paise,
       COALESCE(array_agg(x.account_id ORDER BY c.credit_id) FILTER (WHERE x.id IS NOT NULL), '{}') AS credit_account_ids,
       COALESCE(sum(x.amount_paise), 0)::bigint AS credited_paise
FROM finance.transfers t
JOIN finance.transactions d ON d.id = t.debit_id
LEFT JOIN finance.transfer_credits c ON c.transfer_id = t.id
LEFT JOIN finance.transactions x ON x.id = c.credit_id";

impl From<PartsRow> for Parts {
    fn from(r: PartsRow) -> Self {
        Self {
            id: r.id,
            rung: r.rung,
            debit_account_id: r.debit_account_id,
            debit_paise: r.debit_paise,
            credit_account_ids: r.credit_account_ids,
            credited_paise: r.credited_paise,
        }
    }
}

/// The display figures of these Transfers.
///
/// # Errors
/// The database failed.
pub async fn parts(db: impl PgExecutor<'_>, ids: &[i64]) -> Result<Vec<Parts>, sqlx::Error> {
    let rows: Vec<PartsRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{PARTS} WHERE t.id = ANY($1) GROUP BY t.id, d.account_id, d.amount_paise"
    )))
    .bind(ids)
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

/// The display figures of one Transfer.
///
/// # Errors
/// The database failed.
pub async fn part(db: impl PgExecutor<'_>, id: i64) -> Result<Option<Parts>, sqlx::Error> {
    Ok(parts(db, &[id]).await?.pop())
}

/// The ids of the credits linked to a Transfer.
///
/// # Errors
/// The database failed.
pub async fn credit_ids(db: impl PgExecutor<'_>, id: i64) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT credit_id FROM finance.transfer_credits WHERE transfer_id = $1 ORDER BY credit_id",
    )
    .bind(id)
    .fetch_all(db)
    .await
}
