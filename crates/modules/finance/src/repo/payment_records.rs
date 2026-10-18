//! `finance.payment_records`: what a payment app noted. Never money on its own.

use chrono::NaiveDateTime;
use sqlx::PgExecutor;

use crate::reader::contract::PaymentRecord;

/// A stored Payment record.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PaymentRecordRow {
    /// Row id.
    pub id: i64,
    /// The Payment history it came from.
    pub statement_id: i64,
    /// Local time of the payment.
    pub occurred_at: NaiveDateTime,
    /// With whom.
    pub description: String,
    /// Amount, always above 0.
    pub amount_paise: i64,
    /// `debit` or `credit`.
    pub direction: String,
    /// The app's own id.
    pub source_id: Option<String>,
    /// The bank reference.
    pub utr: Option<String>,
    /// The funding account as printed.
    pub instrument: Option<String>,
    /// The issuer recognised in the instrument.
    pub instrument_issuer: Option<String>,
    /// The trailing visible digits of the instrument.
    pub instrument_digits: Option<String>,
    /// The Transaction it links to, once linked.
    pub transaction_id: Option<i64>,
}

const COLUMNS: &str = "id, statement_id, occurred_at, description, amount_paise, direction, source_id, utr, instrument, instrument_issuer, instrument_digits, transaction_id";

/// Stores a record unless one with the same `source_id` exists; `false` when it was a repeat.
///
/// # Errors
/// The database failed.
pub async fn insert(
    db: impl PgExecutor<'_>,
    statement_id: i64,
    record: &PaymentRecord,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query(
        "INSERT INTO finance.payment_records
           (statement_id, occurred_at, description, amount_paise, direction, source_id, utr, instrument,
            instrument_issuer, instrument_digits)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (source_id) WHERE source_id IS NOT NULL DO NOTHING",
    )
    .bind(statement_id)
    .bind(record.occurred_at)
    .bind(&record.description)
    .bind(record.amount_paise)
    .bind(record.direction.id())
    .bind(&record.source_id)
    .bind(&record.utr)
    .bind(&record.instrument)
    .bind(&record.instrument_issuer)
    .bind(&record.instrument_digits)
    .execute(db)
    .await?;
    Ok(done.rows_affected() == 1)
}

/// Records waiting for their Transaction, oldest first.
///
/// # Errors
/// The database failed.
pub async fn unlinked(db: impl PgExecutor<'_>) -> Result<Vec<PaymentRecordRow>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM finance.payment_records WHERE transaction_id IS NULL ORDER BY occurred_at, id"
    )))
    .fetch_all(db)
    .await
}

/// The record linked to a Transaction, if any.
///
/// # Errors
/// The database failed.
pub async fn for_transaction(
    db: impl PgExecutor<'_>,
    transaction_id: i64,
) -> Result<Option<PaymentRecordRow>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM finance.payment_records WHERE transaction_id = $1"
    )))
    .bind(transaction_id)
    .fetch_optional(db)
    .await
}

/// Links a record to a Transaction. The database refuses a second record for one
/// Transaction.
///
/// # Errors
/// The database failed, or the Transaction already has a record.
pub async fn link(
    db: impl PgExecutor<'_>,
    record_id: i64,
    transaction_id: i64,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query("UPDATE finance.payment_records SET transaction_id = $2 WHERE id = $1")
        .bind(record_id)
        .bind(transaction_id)
        .execute(db)
        .await?;
    Ok(done.rows_affected() == 1)
}

/// Every stored record, for telling new records from ones already stored.
///
/// # Errors
/// The database failed.
pub async fn all(db: impl PgExecutor<'_>) -> Result<Vec<PaymentRecordRow>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM finance.payment_records ORDER BY id"
    )))
    .fetch_all(db)
    .await
}
