//! The Transactions the Overview adds up: the same rows as the Transactions page, with what
//! each one's Transfer needs.

use chrono::NaiveDate;
use sqlx::PgExecutor;

use super::ledger_sql::{FROM, WITH_TRANSFERS, bind_filter, where_clause};
use crate::{
    ledger::{Filter, Membership, Row},
    reader::contract::Direction,
};

#[derive(sqlx::FromRow)]
struct Record {
    date: NaiveDate,
    #[sqlx(try_from = "String")]
    direction: Direction,
    amount_paise: i64,
    transfer_id: Option<i64>,
    debit_paise: Option<i64>,
    credited_paise: Option<i64>,
    carries_extra: bool,
}

impl From<Record> for Row {
    fn from(r: Record) -> Self {
        let transfer = r.transfer_id.zip(r.debit_paise).zip(r.credited_paise).map(
            |((transfer_id, debit_paise), credited_paise)| Membership {
                transfer_id,
                debit_paise,
                credited_paise,
                carries_extra: r.carries_extra,
            },
        );
        Self {
            date: r.date,
            direction: r.direction,
            amount_paise: r.amount_paise,
            transfer,
        }
    }
}

/// Every Transaction of `filter` (its direction is ignored: the sums split by it themselves).
///
/// # Errors
/// The database failed.
pub async fn rows(db: impl PgExecutor<'_>, filter: &Filter) -> Result<Vec<Row>, sqlx::Error> {
    let records: Vec<Record> = bind_filter(
        sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "{WITH_TRANSFERS}
         SELECT t.date, t.direction, t.amount_paise, m.transfer_id, tr.debit_paise,
                tr.credited_paise, COALESCE(t.id = tr.last_credit_id, false) AS carries_extra
         {FROM} {}",
            where_clause(None)
        ))),
        filter,
    )
    .fetch_all(db)
    .await?;
    Ok(records.into_iter().map(Row::from).collect())
}
