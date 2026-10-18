//! The `finance.transactions` query behind Read again and matching: what is stored.

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
}

/// The Account's stored rows as matching sees them (their resolved view), in id order. All
/// of the Account's rows are compared, not only one Statement's, because overlapping
/// Statements share rows.
///
/// # Errors
/// The database failed.
pub async fn stored_keys(
    db: impl PgExecutor<'_>,
    account_id: i64,
) -> Result<Vec<StoredEvidence>, sqlx::Error> {
    let rows: Vec<KeyRow> = sqlx::query_as(
        "SELECT id, date, amount_paise, direction, source_id, balance_paise, reference, description
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
            })
        })
        .collect())
}
