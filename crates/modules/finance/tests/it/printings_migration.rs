//! The Printings migration gives every existing Transaction its first Printing.

use sqlx::migrate::Migrator;
use vitarchy_core::test_db::TestDb;

const PRINTINGS_VERSION: i64 = 20_260_505_000_000;

#[tokio::test]
async fn backfills_one_printing_per_existing_transaction_from_its_own_columns() {
    let db = TestDb::create("fin_pr_migration").await;
    let module = vitarchy_finance::module();
    let all = module.migrator();
    let before: Vec<_> = all
        .iter()
        .filter(|m| m.version < PRINTINGS_VERSION)
        .cloned()
        .collect();
    let mut old = Migrator::with_migrations(before);
    old.set_ignore_missing(true);
    old.run(&db.pool).await.unwrap();
    seed(&db).await;

    all.run(&db.pool).await.unwrap();

    let rows: Vec<(String, i64, bool, Option<i64>)> = sqlx::query_as(
        "SELECT p.description, p.statement_id - t.statement_id, p.gone_at IS NOT NULL, p.balance_paise
         FROM finance.printings p JOIN finance.transactions t ON t.id = p.transaction_id
         ORDER BY t.id",
    )
    .fetch_all(&db.pool)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![
            ("first row".to_owned(), 0, false, Some(900)),
            ("second row".to_owned(), 0, true, None),
        ]
    );
    db.drop().await;
}

async fn seed(db: &TestDb) {
    sqlx::raw_sql(
        "INSERT INTO finance.accounts (issuer, kind, name) VALUES ('Bank', 'savings', 'n');
         INSERT INTO finance.statements (file_sha256, file_name, outcome) VALUES (repeat('a', 64), 'f', 'read');
         INSERT INTO finance.transactions (account_id, statement_id, date, description, amount_paise, direction, balance_paise, no_longer_found_at)
         SELECT a.id, s.id, '2026-07-01', 'first row', 100, 'debit', 900, NULL FROM finance.accounts a, finance.statements s;
         INSERT INTO finance.transactions (account_id, statement_id, date, description, amount_paise, direction, balance_paise, no_longer_found_at)
         SELECT a.id, s.id, '2026-07-02', 'second row', 200, 'credit', NULL, now() FROM finance.accounts a, finance.statements s;",
    )
    .execute(&db.pool)
    .await
    .unwrap();
}
