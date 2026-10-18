//! Writing a Transaction's resolved view (ADR 0015). Amount, date and direction are never
//! touched.

use sqlx::PgExecutor;

use crate::printings::Resolved;

/// Stores `resolved` as the Transaction's columns. A Transaction found by no Printing gets
/// its `no_longer_found_at` mark (kept if already set); otherwise the mark is cleared.
///
/// # Errors
/// The database failed.
pub async fn apply(
    db: impl PgExecutor<'_>,
    id: i64,
    resolved: &Resolved,
) -> Result<(), sqlx::Error> {
    let d = &resolved.detail;
    sqlx::query(
        "UPDATE finance.transactions
         SET description = $2, balance_paise = $3, reference = $4, source_id = $5,
             foreign_amount = $6, foreign_currency = $7,
             no_longer_found_at = CASE WHEN $8 THEN COALESCE(no_longer_found_at, now()) END
         WHERE id = $1",
    )
    .bind(id)
    .bind(&d.description)
    .bind(d.balance_paise)
    .bind(&d.reference)
    .bind(&d.source_id)
    .bind(&d.foreign_amount)
    .bind(&d.foreign_currency)
    .bind(resolved.no_longer_found)
    .execute(db)
    .await?;
    Ok(())
}
