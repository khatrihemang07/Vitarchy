//! What Transfer detection reads: the own, non-hidden Accounts and their Transactions with
//! every Printing's description and reference.

use chrono::NaiveDate;
use sqlx::PgConnection;

use super::printings;
use crate::{
    accounts::visible_digits,
    reader::contract::{AccountKind, Direction},
    transfers::{AccountInfo, Candidate},
};

#[derive(sqlx::FromRow)]
struct AccountRow {
    id: i64,
    kind: String,
    mask: Option<String>,
    remembered: Vec<String>,
}

/// The Accounts that are not hidden, with every run of trailing digits each is known by.
///
/// # Errors
/// The database failed.
pub async fn accounts(db: &mut PgConnection) -> Result<Vec<AccountInfo>, sqlx::Error> {
    let rows: Vec<AccountRow> = sqlx::query_as(
        "SELECT a.id, a.kind, a.mask,
                COALESCE(array_agg(i.digits) FILTER (WHERE i.digits IS NOT NULL), '{}') AS remembered
         FROM finance.accounts a
         LEFT JOIN finance.account_identifiers i ON i.account_id = a.id
         WHERE NOT a.hidden GROUP BY a.id ORDER BY a.id",
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            let own = r
                .mask
                .as_deref()
                .map(visible_digits)
                .filter(|d| !d.is_empty());
            Some(AccountInfo {
                id: r.id,
                kind: AccountKind::from_id(&r.kind)?,
                digits: own.into_iter().chain(r.remembered).collect(),
            })
        })
        .collect())
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    account_id: i64,
    date: NaiveDate,
    amount_paise: i64,
    #[sqlx(try_from = "String")]
    direction: Direction,
    description: String,
    reference: Option<String>,
}

/// The Transactions of non-hidden Accounts dated inside `range` (all when `None`) that a Read
/// again still finds, each with its own and every Printing's description and reference.
///
/// # Errors
/// The database failed.
pub async fn candidates(
    db: &mut PgConnection,
    range: Option<(NaiveDate, NaiveDate)>,
) -> Result<Vec<Candidate>, sqlx::Error> {
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT t.id, t.account_id, t.date, t.amount_paise, t.direction, t.description, t.reference
         FROM finance.transactions t JOIN finance.accounts a ON a.id = t.account_id
         WHERE NOT a.hidden AND t.no_longer_found_at IS NULL
           AND ($1::date IS NULL OR t.date BETWEEN $1 AND $2)
         ORDER BY t.date, t.id",
    )
    .bind(range.map(|(from, _)| from))
    .bind(range.map(|(_, to)| to))
    .fetch_all(&mut *db)
    .await?;
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let mut printed = printings::for_transactions(&mut *db, &ids).await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let mut texts = vec![row.description];
            let mut references: Vec<String> = row.reference.into_iter().collect();
            for printing in printed.remove(&row.id).unwrap_or_default() {
                push_new(&mut texts, Some(printing.printed.description));
                push_new(&mut references, printing.printed.reference);
            }
            Candidate {
                id: row.id,
                account_id: row.account_id,
                direction: row.direction,
                amount_paise: row.amount_paise,
                date: row.date,
                texts,
                references,
            }
        })
        .collect())
}

fn push_new(list: &mut Vec<String>, item: Option<String>) {
    if let Some(item) = item.filter(|i| !i.trim().is_empty() && !list.contains(i)) {
        list.push(item);
    }
}
