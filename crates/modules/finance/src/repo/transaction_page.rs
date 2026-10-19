//! A page of Transactions, for the Transactions table.

use chrono::{NaiveDate, NaiveTime};
use sqlx::PgExecutor;

use crate::reader::contract::Direction;

/// Which Transactions, and which page of them.
#[derive(Debug, Clone, Copy)]
pub struct Filter {
    /// Only this Account.
    pub account_id: Option<i64>,
    /// Only on or after this date.
    pub from: Option<NaiveDate>,
    /// Only on or before this date.
    pub to: Option<NaiveDate>,
}

/// A Transaction as the table shows it.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PageRow {
    /// Row id.
    pub id: i64,
    /// Its Account.
    pub account_id: i64,
    /// The printed date.
    pub date: NaiveDate,
    /// The local time, when any Statement printed one.
    pub time: Option<NaiveTime>,
    /// The printed description.
    pub description: String,
    /// Amount, always above 0.
    pub amount_paise: i64,
    /// Which way the money moved.
    #[sqlx(try_from = "String")]
    pub direction: Direction,
    /// Balance after, when printed.
    pub balance_paise: Option<i64>,
    /// Foreign amount as printed.
    pub foreign_amount: Option<String>,
    /// Foreign currency code.
    pub foreign_currency: Option<String>,
    /// A UPI or NEFT reference.
    pub reference: Option<String>,
    /// Whether a Read again no longer finds it.
    pub no_longer_found: bool,
    /// How many Statements printed it (Printings, gone ones included).
    pub printing_count: i64,
    /// The Transfer it is part of, when it is.
    pub transfer_id: Option<i64>,
}

/// Rows of hidden Accounts never show.
const FROM: &str = "FROM finance.transactions t
JOIN finance.accounts a ON a.id = t.account_id AND NOT a.hidden";

const WHERE: &str = "WHERE ($1::bigint IS NULL OR t.account_id = $1)
  AND ($2::date IS NULL OR t.date >= $2) AND ($3::date IS NULL OR t.date <= $3)";

/// The page of rows, newest first.
///
/// # Errors
/// The database failed.
pub async fn page(
    db: impl PgExecutor<'_>,
    filter: Filter,
    limit: i64,
    offset: i64,
) -> Result<Vec<PageRow>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT t.id, t.account_id, t.date, t.time, t.description, t.amount_paise, t.direction,
                t.balance_paise, t.foreign_amount, t.foreign_currency, t.reference,
                t.no_longer_found_at IS NOT NULL AS no_longer_found,
                (SELECT count(*) FROM finance.printings g WHERE g.transaction_id = t.id) AS printing_count,
                COALESCE((SELECT x.id FROM finance.transfers x WHERE x.debit_id = t.id),
                         (SELECT c.transfer_id FROM finance.transfer_credits c WHERE c.credit_id = t.id)) AS transfer_id
         {FROM}
         {WHERE} ORDER BY t.date DESC, t.id DESC LIMIT $4 OFFSET $5"
    )))
    .bind(filter.account_id)
    .bind(filter.from)
    .bind(filter.to)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
}

/// How many rows the filter matches in all.
///
/// # Errors
/// The database failed.
pub async fn total(db: impl PgExecutor<'_>, filter: Filter) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
        "SELECT count(*) {FROM} {WHERE}"
    )))
    .bind(filter.account_id)
    .bind(filter.from)
    .bind(filter.to)
    .fetch_one(db)
    .await
}
