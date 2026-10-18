//! The writes that merge one Account into another (see [`crate::merge`]). Each moves rows and
//! never loses one; only the emptied Account row and the folded Transaction rows are deleted.

use sqlx::PgConnection;

/// Points the Transactions at an Account.
///
/// # Errors
/// The database failed.
pub async fn repoint_transactions(
    db: &mut PgConnection,
    ids: &[i64],
    account_id: i64,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query("UPDATE finance.transactions SET account_id = $2 WHERE id = ANY($1)")
        .bind(ids)
        .bind(account_id)
        .execute(db)
        .await?;
    Ok(done.rows_affected())
}

/// Folds a Transaction into another: its Printings and Payment record link move onto `into_id`,
/// then the emptied Transaction row goes. A Printing from a Statement `into_id` already has
/// is dropped, and a link is kept only when `into_id` has none (the record then waits to link
/// again). Returns how many Printings moved.
///
/// # Errors
/// The database failed.
pub async fn fold_transaction(
    db: &mut PgConnection,
    from_id: i64,
    into_id: i64,
) -> Result<u64, sqlx::Error> {
    let moved = sqlx::query(
        "UPDATE finance.printings p SET transaction_id = $2
         WHERE p.transaction_id = $1 AND NOT EXISTS (
             SELECT 1 FROM finance.printings q
             WHERE q.transaction_id = $2 AND q.statement_id = p.statement_id)",
    )
    .bind(from_id)
    .bind(into_id)
    .execute(&mut *db)
    .await?
    .rows_affected();
    sqlx::query("DELETE FROM finance.printings WHERE transaction_id = $1")
        .bind(from_id)
        .execute(&mut *db)
        .await?;
    sqlx::query(
        "UPDATE finance.payment_records SET transaction_id = $2
         WHERE transaction_id = $1
           AND NOT EXISTS (SELECT 1 FROM finance.payment_records WHERE transaction_id = $2)",
    )
    .bind(from_id)
    .bind(into_id)
    .execute(&mut *db)
    .await?;
    sqlx::query(
        "UPDATE finance.payment_records SET transaction_id = NULL WHERE transaction_id = $1",
    )
    .bind(from_id)
    .execute(&mut *db)
    .await?;
    sqlx::query("DELETE FROM finance.transactions WHERE id = $1")
        .bind(from_id)
        .execute(&mut *db)
        .await?;
    Ok(moved)
}

/// Moves an Account's Statement slots and remembered identifiers to another Account, then
/// deletes the emptied Account row.
///
/// # Errors
/// The database failed, or the Account still has Transactions.
pub async fn absorb_account(
    db: &mut PgConnection,
    from_id: i64,
    keep_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE finance.statement_accounts SET account_id = $2 WHERE account_id = $1")
        .bind(from_id)
        .bind(keep_id)
        .execute(&mut *db)
        .await?;
    sqlx::query(
        "INSERT INTO finance.account_identifiers (account_id, issuer, kind, digits)
         SELECT $2, issuer, kind, digits FROM finance.account_identifiers WHERE account_id = $1
         ON CONFLICT (account_id, issuer, kind, digits) DO NOTHING",
    )
    .bind(from_id)
    .bind(keep_id)
    .execute(&mut *db)
    .await?;
    sqlx::query("DELETE FROM finance.accounts WHERE id = $1")
        .bind(from_id)
        .execute(&mut *db)
        .await?;
    Ok(())
}
