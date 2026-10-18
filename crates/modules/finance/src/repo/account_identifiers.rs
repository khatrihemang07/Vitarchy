//! `finance.account_identifiers`: the extra (issuer, kind, trailing digits) triples an Account is
//! also known by, remembered from the person's reassignments.

use sqlx::PgExecutor;

use crate::reader::contract::AccountKind;

/// Remembers that this issuer, kind and trailing digits mean `account_id`; repeats change nothing.
///
/// # Errors
/// The database failed.
pub async fn remember(
    db: impl PgExecutor<'_>,
    account_id: i64,
    issuer: &str,
    kind: AccountKind,
    digits: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO finance.account_identifiers (account_id, issuer, kind, digits)
         VALUES ($1, $2, $3, $4) ON CONFLICT (account_id, issuer, kind, digits) DO NOTHING",
    )
    .bind(account_id)
    .bind(issuer)
    .bind(kind.id())
    .bind(digits)
    .execute(db)
    .await?;
    Ok(())
}
