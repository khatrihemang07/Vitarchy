//! An Account with what its Transactions add up to, for the Accounts list.

use chrono::NaiveDate;
use sqlx::PgExecutor;

/// An Account and its figures. Rows a Read again no longer finds are left out of them.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AccountStats {
    /// Row id.
    pub id: i64,
    /// The issuer's name.
    pub issuer: String,
    /// `savings`, `credit_card` or `loan`.
    pub kind: String,
    /// The number as printed.
    pub mask: Option<String>,
    /// The name the person sees.
    pub name: String,
    /// Balance after the latest-dated Transaction that printed one.
    pub latest_balance_paise: Option<i64>,
    /// Transactions stored.
    pub transaction_count: i64,
    /// Date of the earliest Transaction.
    pub first_date: Option<NaiveDate>,
    /// Date of the latest Transaction.
    pub last_date: Option<NaiveDate>,
    /// Confirmed Statements that list this Account.
    pub statement_count: i64,
}

const SELECT: &str = "
SELECT a.id, a.issuer, a.kind, a.mask, a.name,
  (SELECT t.balance_paise FROM finance.transactions t
    WHERE t.account_id = a.id AND t.balance_paise IS NOT NULL AND t.no_longer_found_at IS NULL
    ORDER BY t.date DESC, t.id DESC LIMIT 1) AS latest_balance_paise,
  (SELECT count(*) FROM finance.transactions t
    WHERE t.account_id = a.id AND t.no_longer_found_at IS NULL) AS transaction_count,
  (SELECT min(t.date) FROM finance.transactions t
    WHERE t.account_id = a.id AND t.no_longer_found_at IS NULL) AS first_date,
  (SELECT max(t.date) FROM finance.transactions t
    WHERE t.account_id = a.id AND t.no_longer_found_at IS NULL) AS last_date,
  (SELECT count(*) FROM finance.statement_accounts sa
    JOIN finance.statements s ON s.id = sa.statement_id AND s.status = 'confirmed'
    WHERE sa.account_id = a.id) AS statement_count
FROM finance.accounts a";

/// Every Account with its figures, oldest first.
///
/// # Errors
/// The database failed.
pub async fn list(db: impl PgExecutor<'_>) -> Result<Vec<AccountStats>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!("{SELECT} ORDER BY a.id")))
        .fetch_all(db)
        .await
}

/// One Account with its figures.
///
/// # Errors
/// The database failed.
pub async fn get(db: impl PgExecutor<'_>, id: i64) -> Result<Option<AccountStats>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!("{SELECT} WHERE a.id = $1")))
        .bind(id)
        .fetch_optional(db)
        .await
}
