//! The Printings of one Transaction with their Statements' file names, for the API.

use chrono::{NaiveDate, NaiveTime};
use sqlx::PgExecutor;

/// One Printing and the Statement that printed it.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PrintingRow {
    /// Printing id.
    pub id: i64,
    /// The Statement that printed it.
    pub statement_id: i64,
    /// That Statement's uploaded file name.
    pub file_name: String,
    /// The date as that Statement printed it.
    pub date: NaiveDate,
    /// The local time as that Statement printed it, if it prints one.
    pub time: Option<NaiveTime>,
    /// The description as printed.
    pub description: String,
    /// Balance after, when printed.
    pub balance_paise: Option<i64>,
    /// A UPI or NEFT reference.
    pub reference: Option<String>,
    /// Whether a Read again no longer finds it.
    pub gone: bool,
}

/// Whether a Transaction exists.
///
/// # Errors
/// The database failed.
pub async fn transaction_exists(db: impl PgExecutor<'_>, id: i64) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM finance.transactions WHERE id = $1)")
        .bind(id)
        .fetch_one(db)
        .await
}

/// A Transaction's Printings, in creation order.
///
/// # Errors
/// The database failed.
pub async fn of_transaction(
    db: impl PgExecutor<'_>,
    transaction_id: i64,
) -> Result<Vec<PrintingRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT p.id, p.statement_id, s.file_name, p.date, p.time, p.description, p.balance_paise,
                p.reference, p.gone_at IS NOT NULL AS gone
         FROM finance.printings p JOIN finance.statements s ON s.id = p.statement_id
         WHERE p.transaction_id = $1 ORDER BY p.id",
    )
    .bind(transaction_id)
    .fetch_all(db)
    .await
}
