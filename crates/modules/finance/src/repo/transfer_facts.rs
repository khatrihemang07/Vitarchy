//! What linking a Transfer by hand needs to know about Transactions: their Account, whether
//! that Account is hidden, and whether they are already in a Transfer.

use chrono::NaiveDate;
use sqlx::PgExecutor;

use crate::{reader::contract::Direction, transfers::Candidate};

/// A Transaction as linking sees it.
// Independent yes/no facts of one SQL row; an enum would only restate them.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Facts {
    /// Transaction id.
    pub id: i64,
    /// Its Account.
    pub account_id: i64,
    /// Money out or Money in.
    #[sqlx(try_from = "String")]
    pub direction: Direction,
    /// Amount, above 0.
    pub amount_paise: i64,
    /// The printed date.
    pub date: NaiveDate,
    /// The shown description.
    pub description: String,
    /// Whether its Account is hidden.
    pub hidden: bool,
    /// Whether it is already in a Transfer.
    pub linked: bool,
    /// Whether its Account is a credit card.
    pub on_card: bool,
    /// Whether a Read again no longer finds it.
    pub no_longer_found: bool,
}

impl Facts {
    /// The same Transaction for the pure rules (which need no descriptions to validate).
    #[must_use]
    pub fn candidate(&self) -> Candidate {
        Candidate {
            id: self.id,
            account_id: self.account_id,
            direction: self.direction,
            amount_paise: self.amount_paise,
            date: self.date,
            texts: Vec::new(),
            references: Vec::new(),
        }
    }
}

const SELECT: &str =
    "SELECT t.id, t.account_id, t.direction, t.amount_paise, t.date, t.description,
       a.hidden,
       (EXISTS (SELECT 1 FROM finance.transfers x WHERE x.debit_id = t.id)
        OR EXISTS (SELECT 1 FROM finance.transfer_credits c WHERE c.credit_id = t.id)) AS linked,
       a.kind = 'credit_card' AS on_card,
       t.no_longer_found_at IS NOT NULL AS no_longer_found
FROM finance.transactions t JOIN finance.accounts a ON a.id = t.account_id";

/// These Transactions' facts (a missing id is simply absent).
///
/// # Errors
/// The database failed.
pub async fn facts(db: impl PgExecutor<'_>, ids: &[i64]) -> Result<Vec<Facts>, sqlx::Error> {
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{SELECT} WHERE t.id = ANY($1) ORDER BY t.id"
    )))
    .bind(ids)
    .fetch_all(db)
    .await
}

/// The Transactions within `days` of `anchor` on another Account that is not hidden, going the
/// other way and still found, nearest date first. Whether each is already in a Transfer is
/// in its facts.
///
/// # Errors
/// The database failed.
pub async fn near(
    db: impl PgExecutor<'_>,
    anchor: &Facts,
    days: i32,
) -> Result<Vec<Facts>, sqlx::Error> {
    let other = match anchor.direction {
        Direction::Debit => Direction::Credit,
        Direction::Credit => Direction::Debit,
    };
    sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{SELECT} WHERE t.direction = $1 AND t.account_id <> $2 AND NOT a.hidden
           AND t.no_longer_found_at IS NULL AND t.date BETWEEN $3 - $4 AND $3 + $4
         ORDER BY abs(t.date - $3), t.id"
    )))
    .bind(other.id())
    .bind(anchor.account_id)
    .bind(anchor.date)
    .bind(days)
    .fetch_all(db)
    .await
}
