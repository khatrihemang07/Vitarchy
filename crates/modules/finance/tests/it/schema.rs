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
            "instrument_accounts",
            "markers",
            "printings",
            "runs",
            "statement_accounts",
            "statement_passwords",
            "statements",
            "transactions",
            "transfer_credits",
            "transfer_rejections",
            "transfers"
        ]
    );
    db.drop().await;
}

#[tokio::test]
async fn transfers_refuse_an_unknown_rung_and_a_credit_in_two_transfers() {
    let db = finance_db("transfer_checks").await;
    sqlx::query(
        "INSERT INTO finance.accounts (issuer, kind, name) VALUES ('Example', 'savings', 'A')",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    let stmt: i64 = sqlx::query_scalar(
        "INSERT INTO finance.statements (file_sha256, file_name, outcome) VALUES (repeat('a', 64), 'a', 'read') RETURNING id",
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    for _ in 1..=3 {
        sqlx::query(
            "INSERT INTO finance.transactions (account_id, statement_id, date, description, amount_paise, direction)
             SELECT id, $1, '2026-07-01', 'x', 100, 'debit' FROM finance.accounts",
        )
        .bind(stmt)
        .execute(&db.pool)
        .await
        .unwrap();
    }
    assert!(
        refused(
            &db,
            "INSERT INTO finance.transfers (debit_id, rung) VALUES (1, 'guess')"
        )
        .await
    );
    sqlx::query(
        "INSERT INTO finance.transfers (debit_id, rung) VALUES (1, 'manual'), (2, 'manual')",
    )
    .execute(&db.pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO finance.transfer_credits (transfer_id, credit_id) SELECT id, 3 FROM finance.transfers WHERE debit_id = 1")
        .execute(&db.pool)
        .await
        .unwrap();
    assert!(refused(&db, "INSERT INTO finance.transfer_credits (transfer_id, credit_id) SELECT id, 3 FROM finance.transfers WHERE debit_id = 2").await);
    assert!(
        refused(
            &db,
            "INSERT INTO finance.transfers (debit_id, rung) VALUES (1, 'manual')"
        )
        .await
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
