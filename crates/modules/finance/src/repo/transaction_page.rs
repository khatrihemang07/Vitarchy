//! A page of Transactions with their linked Payment record, for the Transactions table.

use chrono::{NaiveDate, NaiveDateTime};
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

/// A Transaction and, when linked, its Payment record.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PageRow {
    /// Row id.
    pub id: i64,
    /// Its Account.
    pub account_id: i64,
    /// The printed date.
    pub date: NaiveDate,
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
    /// The linked Payment record's id.
    pub record_id: Option<i64>,
    /// When the linked payment happened.
    pub record_occurred_at: Option<NaiveDateTime>,
    /// The linked Payment record's description.
    pub record_description: Option<String>,
    /// The linked Payment record's bank reference.
    pub record_utr: Option<String>,
    /// The linked Payment record's funding account.
    pub record_instrument: Option<String>,
}

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
        "SELECT t.id, t.account_id, t.date, t.description, t.amount_paise, t.direction,
                t.balance_paise, t.foreign_amount, t.foreign_currency, t.reference,
                t.no_longer_found_at IS NOT NULL AS no_longer_found,
                (SELECT count(*) FROM finance.printings g WHERE g.transaction_id = t.id) AS printing_count,
                p.id AS record_id, p.occurred_at AS record_occurred_at,
                p.description AS record_description, p.utr AS record_utr,
                p.instrument AS record_instrument
         FROM finance.transactions t
         LEFT JOIN finance.payment_records p ON p.transaction_id = t.id
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
        "SELECT count(*) FROM finance.transactions t {WHERE}"
    )))
    .bind(filter.account_id)
    .bind(filter.from)
    .bind(filter.to)
    .fetch_one(db)
    .await
}
