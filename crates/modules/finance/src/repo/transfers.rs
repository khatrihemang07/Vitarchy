//! `finance.transfers` and `finance.transfer_credits`: writing Transfers.

use sqlx::{PgConnection, PgExecutor};

use crate::transfers::Rung;

/// The Transfer of this debit, made with `rung` when it has none.
///
/// # Errors
/// The database failed.
pub async fn get_or_create(
    db: impl PgExecutor<'_>,
    debit_id: i64,
    rung: Rung,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO finance.transfers (debit_id, rung) VALUES ($1, $2)
         ON CONFLICT (debit_id) DO UPDATE SET debit_id = EXCLUDED.debit_id RETURNING id",
    )
    .bind(debit_id)
    .bind(rung.id())
    .fetch_one(db)
    .await
}

/// Links credits to a Transfer.
///
/// # Errors
/// The database failed, or a credit is already in a Transfer.
pub async fn add_credits(
    db: impl PgExecutor<'_>,
    transfer_id: i64,
    credit_ids: &[i64],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO finance.transfer_credits (transfer_id, credit_id)
         SELECT $1, credit_id FROM unnest($2::bigint[]) AS credit_id",
    )
    .bind(transfer_id)
    .bind(credit_ids)
    .execute(db)
    .await?;
    Ok(())
}

/// Unlinks a Transfer: remembers every (debit, credit) pair so detection never links it
/// again, then deletes the Transfer. `false` when there is no such Transfer.
///
/// # Errors
/// The database failed.
pub async fn unlink(db: &mut PgConnection, id: i64) -> Result<bool, sqlx::Error> {
    sqlx::query(
        "INSERT INTO finance.transfer_rejections (debit_id, credit_id)
         SELECT t.debit_id, c.credit_id FROM finance.transfers t
         JOIN finance.transfer_credits c ON c.transfer_id = t.id WHERE t.id = $1
         ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .execute(&mut *db)
    .await?;
    let done = sqlx::query("DELETE FROM finance.transfers WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(done.rows_affected() == 1)
}

/// Takes an Account out of every Transfer: its debit's Transfer is deleted whole, its credits
/// are unlinked (and a Transfer left with none dissolves). Nothing is remembered, so the pair
/// may link again once the Account is shown. Returns how many Transfers were touched.
///
/// # Errors
/// The database failed.
pub async fn leave_account(db: &mut PgConnection, account_id: i64) -> Result<u64, sqlx::Error> {
    let credits = sqlx::query(
        "DELETE FROM finance.transfer_credits
         WHERE credit_id IN (SELECT id FROM finance.transactions WHERE account_id = $1)",
    )
    .bind(account_id)
    .execute(&mut *db)
    .await?
    .rows_affected();
    let debits = sqlx::query(
        "DELETE FROM finance.transfers
         WHERE debit_id IN (SELECT id FROM finance.transactions WHERE account_id = $1)",
    )
    .bind(account_id)
    .execute(db)
    .await?
    .rows_affected();
    Ok(credits + debits)
}

/// Unlinks credits that sit on the same Account as their debit (two Accounts were merged into
/// one); a Transfer left with none dissolves.
///
/// # Errors
/// The database failed.
pub async fn drop_same_account_links(db: impl PgExecutor<'_>) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        "DELETE FROM finance.transfer_credits c USING finance.transfers t,
                finance.transactions d, finance.transactions x
         WHERE t.id = c.transfer_id AND d.id = t.debit_id AND x.id = c.credit_id
           AND d.account_id = x.account_id",
    )
    .execute(db)
    .await?;
    Ok(done.rows_affected())
}
