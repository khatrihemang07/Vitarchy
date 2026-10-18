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

const IDENTICAL_VERSION: i64 = 20_260_506_000_000;

#[tokio::test]
async fn the_identical_printings_migration_keeps_the_earliest_and_anything_different() {
    let db = TestDb::create("fin_pr_identical").await;
    let module = vitarchy_finance::module();
    let all = module.migrator();
    let before: Vec<_> = all
        .iter()
        .filter(|m| m.version < IDENTICAL_VERSION)
        .cloned()
        .collect();
    let mut old = Migrator::with_migrations(before);
    old.set_ignore_missing(true);
    old.run(&db.pool).await.unwrap();
    sqlx::raw_sql(
        "INSERT INTO finance.accounts (issuer, kind, name) VALUES ('Bank', 'savings', 'n');
         INSERT INTO finance.statements (file_sha256, file_name, outcome)
           SELECT repeat(c, 64), 'f', 'read' FROM unnest(ARRAY['a','b','c','d']) c;
         INSERT INTO finance.transactions (account_id, statement_id, date, description, amount_paise, direction)
           SELECT a.id, (SELECT min(id) FROM finance.statements), '2026-07-01', 'row', 100, 'debit'
           FROM finance.accounts a;
         DELETE FROM finance.printings;
         INSERT INTO finance.printings (transaction_id, statement_id, date, description, reference)
           SELECT t.id, s.id, '2026-07-01', 'row', CASE WHEN s.file_sha256 LIKE 'c%' THEN 'R' END
           FROM finance.transactions t, finance.statements s ORDER BY s.id;",
    )
    .execute(&db.pool)
    .await
    .unwrap();

    all.run(&db.pool).await.unwrap();

    let left: Vec<(Option<String>, bool)> =
        sqlx::query_as("SELECT reference, gone_at IS NOT NULL FROM finance.printings ORDER BY id")
            .fetch_all(&db.pool)
            .await
            .unwrap();
    assert_eq!(
        left,
        vec![(None, false), (Some("R".to_owned()), false)],
        "a, b and d are alike (keep a); c differs"
    );
    let flagged: bool =
        sqlx::query_scalar("SELECT no_longer_found_at IS NOT NULL FROM finance.transactions")
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert!(!flagged);
    db.drop().await;
}
