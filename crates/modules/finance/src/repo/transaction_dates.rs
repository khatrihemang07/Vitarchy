//! The span of dates a set of Transactions covers, so detection can look only near what
//! changed.

use chrono::NaiveDate;
use sqlx::PgExecutor;

/// The earliest and latest date of the Transactions a Statement printed; `None` when it
/// printed none.
///
/// # Errors
/// The database failed.
pub async fn of_statement(
    db: impl PgExecutor<'_>,
    statement_id: i64,
) -> Result<Option<(NaiveDate, NaiveDate)>, sqlx::Error> {
    span(
        db,
        "SELECT min(t.date), max(t.date) FROM finance.transactions t
         WHERE t.id IN (SELECT transaction_id FROM finance.printings WHERE statement_id = $1)",
        statement_id,
    )
    .await
}

/// The earliest and latest date of these Accounts' Transactions; `None` when they have none.
///
/// # Errors
/// The database failed.
pub async fn of_accounts(
    db: impl PgExecutor<'_>,
    account_ids: &[i64],
) -> Result<Option<(NaiveDate, NaiveDate)>, sqlx::Error> {
    let (from, to): (Option<NaiveDate>, Option<NaiveDate>) = sqlx::query_as(
        "SELECT min(date), max(date) FROM finance.transactions WHERE account_id = ANY($1)",
    )
    .bind(account_ids)
    .fetch_one(db)
    .await?;
    Ok(from.zip(to))
}

async fn span(
    db: impl PgExecutor<'_>,
    sql: &str,
    id: i64,
) -> Result<Option<(NaiveDate, NaiveDate)>, sqlx::Error> {
    let (from, to): (Option<NaiveDate>, Option<NaiveDate>) =
        sqlx::query_as(sqlx::AssertSqlSafe(sql.to_owned()))
            .bind(id)
            .fetch_one(db)
            .await?;
    Ok(from.zip(to))
}
