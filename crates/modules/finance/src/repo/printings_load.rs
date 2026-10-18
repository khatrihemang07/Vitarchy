//! Reading Printings back: by Transaction or by Account, each list in creation order.

use std::collections::HashMap;

use sqlx::PgExecutor;

use crate::printings::{PrintedFields, Printing};

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    transaction_id: i64,
    statement_id: i64,
    date: chrono::NaiveDate,
    description: String,
    balance_paise: Option<i64>,
    reference: Option<String>,
    source_id: Option<String>,
    foreign_amount: Option<String>,
    foreign_currency: Option<String>,
    gone: bool,
}

impl From<Row> for (i64, Printing) {
    fn from(r: Row) -> Self {
        (
            r.transaction_id,
            Printing {
                id: r.id,
                statement_id: r.statement_id,
                printed: PrintedFields {
                    date: r.date,
                    description: r.description,
                    balance_paise: r.balance_paise,
                    reference: r.reference,
                    source_id: r.source_id,
                    foreign_amount: r.foreign_amount,
                    foreign_currency: r.foreign_currency,
                },
                gone: r.gone,
            },
        )
    }
}

const COLUMNS: &str = "id, transaction_id, statement_id, date, description, balance_paise, reference, source_id, foreign_amount, foreign_currency, gone_at IS NOT NULL AS gone";

/// The Printings of these Transactions, each list in creation order.
///
/// # Errors
/// The database failed.
pub async fn for_transactions(
    db: impl PgExecutor<'_>,
    transaction_ids: &[i64],
) -> Result<HashMap<i64, Vec<Printing>>, sqlx::Error> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM finance.printings WHERE transaction_id = ANY($1) ORDER BY id"
    )))
    .bind(transaction_ids)
    .fetch_all(db)
    .await?;
    Ok(group(rows))
}

/// The Printings of every Transaction of an Account, each list in creation order.
///
/// # Errors
/// The database failed.
pub async fn for_account(
    db: impl PgExecutor<'_>,
    account_id: i64,
) -> Result<HashMap<i64, Vec<Printing>>, sqlx::Error> {
    let rows: Vec<Row> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "SELECT {COLUMNS} FROM finance.printings
         WHERE transaction_id IN (SELECT id FROM finance.transactions WHERE account_id = $1)
         ORDER BY id"
    )))
    .bind(account_id)
    .fetch_all(db)
    .await?;
    Ok(group(rows))
}

fn group(rows: Vec<Row>) -> HashMap<i64, Vec<Printing>> {
    let mut grouped: HashMap<i64, Vec<Printing>> = HashMap::new();
    for (transaction_id, printing) in rows.into_iter().map(Into::into) {
        grouped.entry(transaction_id).or_default().push(printing);
    }
    grouped
}
