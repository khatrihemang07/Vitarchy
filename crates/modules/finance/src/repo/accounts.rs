//! `finance.accounts`.

use chrono::{DateTime, Utc};
use sqlx::PgExecutor;

use crate::{
    accounts::{AccountIdentity, last4},
    reader::contract::AccountKind,
};

/// A stored Account.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountRow {
    /// Row id.
    pub id: i64,
    /// The issuer's name.
    pub issuer: String,
    /// `savings`, `credit_card` or `loan`.
    pub kind: String,
    /// The number as printed.
    pub mask: Option<String>,
    /// Its last 4 digits, derived from `mask`.
    pub last4: Option<String>,
    /// The name the person sees.
    pub name: String,
    /// Whether the person hid it (ADR 0017).
    pub hidden: bool,
    /// When it was created.
    pub created_at: DateTime<Utc>,
}

const COLUMNS: &str = "id, issuer, kind, mask, last4, name, hidden, created_at";

/// Creates an Account; `last4` is derived from `mask`.
///
/// # Errors
/// The database failed.
pub async fn insert(
    db: impl PgExecutor<'_>,
    issuer: &str,
    kind: AccountKind,
    mask: Option<&str>,
    name: &str,
) -> Result<AccountRow, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "INSERT INTO finance.accounts (issuer, kind, mask, last4, name) VALUES ($1, $2, $3, $4, $5)
         RETURNING {COLUMNS}"
    )))
    .bind(issuer)
    .bind(kind.id())
    .bind(mask)
    .bind(last4(mask))
    .bind(name)
    .fetch_one(db)
    .await
}

/// One Account.
///
/// # Errors
/// The database failed.
pub async fn get(db: impl PgExecutor<'_>, id: i64) -> Result<Option<AccountRow>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM finance.accounts WHERE id = $1"
    )))
    .bind(id)
    .fetch_optional(db)
    .await
}

/// Every Account, oldest first.
///
/// # Errors
/// The database failed.
pub async fn list(db: impl PgExecutor<'_>) -> Result<Vec<AccountRow>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM finance.accounts ORDER BY id"
    )))
    .fetch_all(db)
    .await
}

/// Renames an Account; `false` when it does not exist.
///
/// # Errors
/// The database failed.
pub async fn rename(db: impl PgExecutor<'_>, id: i64, name: &str) -> Result<bool, sqlx::Error> {
    let done = sqlx::query("UPDATE finance.accounts SET name = $2 WHERE id = $1")
        .bind(id)
        .bind(name)
        .execute(db)
        .await?;
    Ok(done.rows_affected() == 1)
}

/// Hides or shows an Account; `false` when it does not exist.
///
/// # Errors
/// The database failed.
pub async fn set_hidden(
    db: impl PgExecutor<'_>,
    id: i64,
    hidden: bool,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query("UPDATE finance.accounts SET hidden = $2 WHERE id = $1")
        .bind(id)
        .bind(hidden)
        .execute(db)
        .await?;
    Ok(done.rows_affected() == 1)
}

#[derive(sqlx::FromRow)]
struct KnownRow {
    account_id: i64,
    issuer: String,
    kind: String,
    mask: Option<String>,
}

/// Every way an Account is known: its own issuer, kind and mask, plus remembered
/// identifiers. Feeds [`crate::match_account`].
///
/// # Errors
/// The database failed.
pub async fn identities(db: impl PgExecutor<'_>) -> Result<Vec<AccountIdentity>, sqlx::Error> {
    let rows: Vec<KnownRow> = sqlx::query_as(
        "SELECT id AS account_id, issuer, kind, mask FROM finance.accounts
         UNION ALL
         SELECT account_id, issuer, kind, digits AS mask FROM finance.account_identifiers
         ORDER BY account_id",
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            Some(AccountIdentity {
                account_id: row.account_id,
                issuer: row.issuer,
                kind: AccountKind::from_id(&row.kind)?,
                mask: row.mask,
            })
        })
        .collect())
}

/// Every Account by its own issuer, kind and mask (no remembered identifiers), oldest first.
///
/// # Errors
/// The database failed.
pub async fn own_identities(db: impl PgExecutor<'_>) -> Result<Vec<AccountIdentity>, sqlx::Error> {
    let rows: Vec<KnownRow> = sqlx::query_as(
        "SELECT id AS account_id, issuer, kind, mask FROM finance.accounts ORDER BY id",
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            Some(AccountIdentity {
                account_id: row.account_id,
                issuer: row.issuer,
                kind: AccountKind::from_id(&row.kind)?,
                mask: row.mask,
            })
        })
        .collect())
}
