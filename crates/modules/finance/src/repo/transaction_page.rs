//! A page of Transactions, for the Transactions table.

use chrono::{NaiveDate, NaiveTime};
use sqlx::PgExecutor;

use super::ledger_sql::{FROM, WITH_TRANSFERS, bind_filter, where_clause};
use crate::{
    ledger::{Filter, Sort},
    reader::contract::Direction,
};

/// A Transaction as the table shows it.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PageRow {
    /// Row id.
    pub id: i64,
    /// Its Account.
    pub account_id: i64,
    /// The printed date.
    pub date: NaiveDate,
    /// The local time, when any Statement printed one.
    pub time: Option<NaiveTime>,
    /// The printed description.
    pub description: String,
    /// Amount, always above 0.
    pub amount_paise: i64,
    /// Which way the money moved.
    #[sqlx(try_from = "String")]
    pub direction: Direction,
    /// Balance after, when printed.
    pub balance_paise: Option<i64>,
    /// Foreign amount as printed.
    pub foreign_amount: Option<String>,
    /// Foreign currency code.
    pub foreign_currency: Option<String>,
    /// A UPI or NEFT reference.
    pub reference: Option<String>,
    /// Whether a Read again no longer finds it.
    pub no_longer_found: bool,
    /// How many Statements printed it (Printings, gone ones included).
    pub printing_count: i64,
    /// The Transfer it is part of, when it is.
    pub transfer_id: Option<i64>,
}

/// The page of rows in `sort` order.
///
/// # Errors
/// The database failed.
pub async fn page(
    db: impl PgExecutor<'_>,
    filter: &Filter,
    sort: Sort,
    limit: i64,
    offset: i64,
) -> Result<Vec<PageRow>, sqlx::Error> {
    bind_filter(
        sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "{WITH_TRANSFERS}
         SELECT t.id, t.account_id, t.date, t.time, t.description, t.amount_paise, t.direction,
                t.balance_paise, t.foreign_amount, t.foreign_currency, t.reference,
                t.no_longer_found_at IS NOT NULL AS no_longer_found,
                (SELECT count(*) FROM finance.printings g WHERE g.transaction_id = t.id) AS printing_count,
                m.transfer_id
         {FROM}
         {} ORDER BY {} LIMIT $5 OFFSET $6",
        where_clause(filter.flow),
        sort.order_by()
    ))),
        filter,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
}

/// How many rows the filter matches in all.
///
/// # Errors
/// The database failed.
pub async fn total(db: impl PgExecutor<'_>, filter: &Filter) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = bind_filter(
        sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "{WITH_TRANSFERS} SELECT count(*) {FROM} {}",
            where_clause(filter.flow)
        ))),
        filter,
    )
    .fetch_one(db)
    .await?;
    Ok(count)
}
