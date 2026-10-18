//! The `finance.transactions` query behind Read again and matching: what is stored.

use std::collections::HashMap;

use sqlx::PgConnection;

use crate::{
    dedupe::{Evidence, Origin, Origins, StoredEvidence},
    reader::contract::Direction,
};

#[derive(sqlx::FromRow)]
struct KeyRow {
    id: i64,
    date: chrono::NaiveDate,
    amount_paise: i64,
    direction: String,
    source_id: Option<String>,
    balance_paise: Option<i64>,
    reference: Option<String>,
    description: String,
    ledger: bool,
    payment_app: bool,
}

#[derive(sqlx::FromRow)]
struct PrintedRow {
    transaction_id: i64,
    date: chrono::NaiveDate,
    source_id: Option<String>,
    balance_paise: Option<i64>,
    reference: Option<String>,
    description: String,
    kind: Option<String>,
}

/// The Account's stored rows as matching sees them, in id order: their resolved view (and the
/// kinds of file whose live Printings make them), and what each Printing printed. All of the
/// Account's rows are compared, not only one Statement's, because overlapping Statements
/// share rows.
///
/// Every Printing of a Transaction is included, gone ones too, on purpose: a row that comes
/// back after its Statement stopped printing it must find its Transaction and revive it
/// (ADR 0016), not store a second one. Origins count live Printings only.
///
/// # Errors
/// The database failed.
pub async fn stored_keys(
    db: &mut PgConnection,
    account_id: i64,
) -> Result<Vec<StoredEvidence>, sqlx::Error> {
    let rows: Vec<KeyRow> = sqlx::query_as(
        "SELECT t.id, t.date, t.amount_paise, t.direction, t.source_id, t.balance_paise,
                t.reference, t.description,
                EXISTS (SELECT 1 FROM finance.printings p JOIN finance.statements s ON s.id = p.statement_id
                        WHERE p.transaction_id = t.id AND p.gone_at IS NULL
                          AND s.kind = 'statement') AS ledger,
                EXISTS (SELECT 1 FROM finance.printings p JOIN finance.statements s ON s.id = p.statement_id
                        WHERE p.transaction_id = t.id AND p.gone_at IS NULL
                          AND s.kind = 'payment_history') AS payment_app
         FROM finance.transactions t WHERE t.account_id = $1 ORDER BY t.id",
    )
    .bind(account_id)
    .fetch_all(&mut *db)
    .await?;
    let printed: Vec<PrintedRow> = sqlx::query_as(
        "SELECT p.transaction_id, p.date, p.source_id, p.balance_paise, p.reference, p.description,
                s.kind
         FROM finance.printings p JOIN finance.transactions t ON t.id = p.transaction_id
         JOIN finance.statements s ON s.id = p.statement_id
         WHERE t.account_id = $1 ORDER BY p.id",
    )
    .bind(account_id)
    .fetch_all(&mut *db)
    .await?;
    let mut by_transaction: HashMap<i64, Vec<PrintedRow>> = HashMap::new();
    for row in printed {
        by_transaction
            .entry(row.transaction_id)
            .or_default()
            .push(row);
    }
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            let direction = Direction::from_id(&r.direction)?;
            let evidence_of =
                |date, source_id: Option<&str>, balance, reference: Option<&str>, text: &str| {
                    Evidence::new(
                        date,
                        r.amount_paise,
                        direction,
                        source_id,
                        balance,
                        reference,
                        text,
                    )
                };
            let printed = by_transaction
                .remove(&r.id)
                .unwrap_or_default()
                .iter()
                .map(|p| {
                    evidence_of(
                        p.date,
                        p.source_id.as_deref(),
                        p.balance_paise,
                        p.reference.as_deref(),
                        &p.description,
                    )
                    .with_origins(origins_of(p.kind.as_deref()))
                })
                .collect();
            let origins = match (r.ledger, r.payment_app) {
                (true, true) => Origins::Both,
                (true, false) => Origins::of(Origin::Statement),
                (false, true) => Origins::of(Origin::PaymentHistory),
                (false, false) => Origins::Unknown,
            };
            let resolved = evidence_of(
                r.date,
                r.source_id.as_deref(),
                r.balance_paise,
                r.reference.as_deref(),
                &r.description,
            )
            .with_origins(origins);
            Some(StoredEvidence {
                id: r.id,
                evidence: resolved,
                printed,
            })
        })
        .collect())
}

/// The origin a Printing carries from the kind of its Statement.
fn origins_of(kind: Option<&str>) -> Origins {
    match kind {
        Some("statement") => Origins::of(Origin::Statement),
        Some("payment_history") => Origins::of(Origin::PaymentHistory),
        _ => Origins::Unknown,
    }
}
