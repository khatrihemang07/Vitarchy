//! The migrations' typed columns and CHECK constraints refuse bad rows.

use crate::common::finance_db;

async fn refused(db: &vitarchy_core::test_db::TestDb, sql: &'static str) -> bool {
    sqlx::query(sql).execute(&db.pool).await.is_err()
}

#[tokio::test]
async fn creates_the_finance_schema_tables() {
    let db = finance_db("tables").await;
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'finance' ORDER BY 1",
    )
    .fetch_all(&db.pool)
    .await
    .unwrap();
    assert_eq!(
        tables,
        [
            "account_identifiers",
            "accounts",
            "payment_records",
            "runs",
            "statement_accounts",
            "statement_passwords",
            "statements",
            "transactions"
        ]
    );
    db.drop().await;
}

#[tokio::test]
async fn check_constraints_refuse_bad_values() {
    let db = finance_db("checks").await;
    assert!(
        refused(
            &db,
            "INSERT INTO finance.accounts (issuer, kind, name) VALUES ('x', 'brokerage', 'n')"
        )
        .await
    );
    assert!(refused(&db, "INSERT INTO finance.accounts (issuer, kind, name, last4) VALUES ('x', 'loan', 'n', '12')").await);
    assert!(refused(&db, "INSERT INTO finance.statements (file_sha256, file_name, outcome) VALUES ('short', 'f', 'read')").await);
    assert!(refused(&db, "INSERT INTO finance.statements (file_sha256, file_name, outcome) VALUES (repeat('a', 64), 'f', 'bogus')").await);
    assert!(refused(&db, "INSERT INTO finance.statements (file_sha256, file_name, outcome, status) VALUES (repeat('a', 64), 'f', 'read', 'confirmed')").await);
    assert!(refused(&db, "INSERT INTO finance.runs (kind, started_at, finished_at, outcome) VALUES ('mystery', now(), now(), 'success')").await);
    db.drop().await;
}

#[tokio::test]
async fn transactions_must_have_positive_amounts_and_a_direction() {
    let db = finance_db("txchecks").await;
    let account = crate::common::account(&db, "XX2003").await;
    let statement = crate::common::statement(&db, 'a').await;
    let insert = |amount: i64, direction: &'static str| {
        let pool = db.pool.clone();
        async move {
            sqlx::query(
                "INSERT INTO finance.transactions (account_id, statement_id, date, description, amount_paise, direction)
                 VALUES ($1, $2, '2026-07-01', 'd', $3, $4)",
            )
            .bind(account.id)
            .bind(statement)
            .bind(amount)
            .bind(direction)
            .execute(&pool)
            .await
            .is_ok()
        }
    };
    assert!(insert(1, "debit").await);
    assert!(!insert(0, "debit").await);
    assert!(!insert(-5, "credit").await);
    assert!(!insert(5, "sideways").await);
    db.drop().await;
}
