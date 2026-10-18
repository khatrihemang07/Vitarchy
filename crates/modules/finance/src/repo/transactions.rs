//! `finance.transactions`.

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use sqlx::PgExecutor;

use crate::reader::contract::Transaction;

pub use super::transactions_read_again::stored_keys;

/// A stored Transaction.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TransactionRow {
    /// Row id.
    pub id: i64,
    /// Its Account.
    pub account_id: i64,
    /// The Statement it came from.
    pub statement_id: i64,
    /// The printed date.
    pub date: NaiveDate,
    /// The local time, when any Statement printed one.
    pub time: Option<NaiveTime>,
    /// The printed description.
    pub description: String,
    /// Amount, always above 0.
    pub amount_paise: i64,
    /// `debit` or `credit`.
    pub direction: String,
    /// Balance after, when printed.
    pub balance_paise: Option<i64>,
    /// Foreign amount as printed.
    pub foreign_amount: Option<String>,
    /// Foreign currency code.
    pub foreign_currency: Option<String>,
    /// A UPI or NEFT reference.
    pub reference: Option<String>,
    /// An id the issuer printed.
    pub source_id: Option<String>,
    /// Set when a Read again no longer finds it.
    pub no_longer_found_at: Option<DateTime<Utc>>,
    /// When it was stored.
    pub created_at: DateTime<Utc>,
}

const COLUMNS: &str = "id, account_id, statement_id, date, time, description, amount_paise, direction, balance_paise, foreign_amount, foreign_currency, reference, source_id, no_longer_found_at, created_at";

/// Stores one Transaction under an Account and Statement.
///
/// # Errors
/// The database failed.
pub async fn insert(
    db: impl PgExecutor<'_>,
    account_id: i64,
    statement_id: i64,
    tx: &Transaction,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO finance.transactions
           (account_id, statement_id, date, time, description, amount_paise, direction,
            balance_paise, foreign_amount, foreign_currency, reference, source_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING id",
    )
    .bind(account_id)
    .bind(statement_id)
    .bind(tx.date)
    .bind(tx.time)
    .bind(&tx.description)
    .bind(tx.amount_paise)
    .bind(tx.direction.id())
    .bind(tx.balance_paise)
    .bind(&tx.foreign_amount)
    .bind(&tx.foreign_currency)
    .bind(&tx.reference)
    .bind(&tx.source_id)
    .fetch_one(db)
    .await
}

/// An Account's Transactions in a date range (either end open), newest first.
///
/// # Errors
/// The database failed.
pub async fn list(
    db: impl PgExecutor<'_>,
    account_id: i64,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
) -> Result<Vec<TransactionRow>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM finance.transactions
         WHERE account_id = $1 AND ($2::date IS NULL OR date >= $2) AND ($3::date IS NULL OR date <= $3)
         ORDER BY date DESC, id DESC"
    )))
    .bind(account_id)
    .bind(from)
    .bind(to)
    .fetch_all(db)
    .await
}
