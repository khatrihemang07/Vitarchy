//! A stage database as it was before payment apps printed Transactions, to be migrated.

use sqlx::migrate::Migrator;
use vitarchy_core::test_db::TestDb;

const RETIRE_VERSION: i64 = 20_260_507_000_000;

/// A database migrated up to just before the payment-app migration, and the Module whose
/// migrator finishes the job.
pub async fn old_database(label: &str) -> (TestDb, vitarchy_core::Module) {
    let db = TestDb::create(label).await;
    let module = vitarchy_finance::module();
    let all = module.migrator();
    let before: Vec<_> = all
        .iter()
        .filter(|m| m.version < RETIRE_VERSION)
        .cloned()
        .collect();
    let mut old = Migrator::with_migrations(before);
    old.set_ignore_missing(true);
    old.run(&db.pool).await.unwrap();
    (db, module)
}

/// Two savings Accounts, a bank Statement with three Transactions (the first two on Account
/// 1, the third on Account 2), and two payment histories: `fully` and `partly` converted.
pub async fn seed_two_histories(db: &TestDb) {
    sqlx::raw_sql(
        "INSERT INTO finance.accounts (issuer, kind, name, mask) VALUES
           ('Bank', 'savings', 'one', 'XXXXXX1111'), ('Bank', 'savings', 'two', 'XXXXXX2222');
         INSERT INTO finance.statements (file_sha256, file_name, kind, outcome, status, confirmed_at, result) VALUES
           (repeat('a', 64), 'bank.pdf', 'statement', 'read', 'confirmed', now(), '{}'),
           (repeat('b', 64), 'fully.csv', 'payment_history', 'read', 'confirmed', now(), '{\"payment_records\":[]}'),
           (repeat('c', 64), 'partly.csv', 'payment_history', 'read', 'confirmed', now(), '{\"payment_records\":[]}');
         INSERT INTO finance.transactions (account_id, statement_id, date, description, amount_paise, direction, reference) VALUES
           (1, 1, '2026-07-01', 'UPI/DR/512345678901/SHOP', 500, 'debit', NULL),
           (1, 1, '2026-07-02', 'UPI/DR/OTHER', 700, 'debit', 'R7'),
           (2, 1, '2026-07-03', 'NEFT/RENT', 900, 'debit', NULL);
         DELETE FROM finance.printings;
         INSERT INTO finance.printings (transaction_id, statement_id, date, description, reference)
           SELECT id, 1, date, description, reference FROM finance.transactions;
         INSERT INTO finance.payment_records
           (statement_id, occurred_at, description, amount_paise, direction, source_id, utr, instrument, transaction_id) VALUES
           (2, '2026-07-01 11:20:00', 'Paid to SHOP', 500, 'debit', 'T1', '512345678901', 'XXXXXX1111', 1),
           (2, '2026-07-02 09:05:00', 'Paid to OTHER', 700, 'debit', 'T2', NULL, 'XXXXXX1111', 2),
           (3, '2026-07-03 10:00:00', 'Paid to LANDLORD', 900, 'debit', 'T3', 'U3', 'ICICI  R', 3),
           (3, '2026-07-04 10:00:00', 'Paid to NOBODY', 100, 'debit', 'T4', NULL, 'ICICI R', NULL);",
    )
    .execute(&db.pool)
    .await
    .unwrap();
}

/// Two savings Accounts, a bank Statement printing one Transaction on each, and a fully
/// linked payment history (instrument `XXXXXX1111` paid from Account 1, `ICICI R` from 2).
pub async fn seed_one_history(db: &TestDb) {
    sqlx::raw_sql(
        "INSERT INTO finance.accounts (issuer, kind, name, mask) VALUES
           ('Bank', 'savings', 'one', 'XXXXXX1111'), ('Bank', 'savings', 'two', 'XXXXXX2222');
         INSERT INTO finance.statements (file_sha256, file_name, kind, outcome, status, confirmed_at, result) VALUES
           (repeat('a', 64), 'bank.pdf', 'statement', 'read', 'confirmed', now(), '{}'),
           (repeat('b', 64), 'fully.csv', 'payment_history', 'read', 'confirmed', now(), '{\"payment_records\":[]}');
         INSERT INTO finance.transactions (account_id, statement_id, date, description, amount_paise, direction, reference) VALUES
           (1, 1, '2026-07-01', 'UPI/DR/512345678901/SHOP', 500, 'debit', NULL),
           (2, 1, '2026-07-03', 'NEFT/RENT', 900, 'debit', NULL);
         DELETE FROM finance.printings;
         INSERT INTO finance.printings (transaction_id, statement_id, date, description, reference)
           SELECT id, 1, date, description, reference FROM finance.transactions;
         INSERT INTO finance.payment_records
           (statement_id, occurred_at, description, amount_paise, direction, source_id, utr, instrument, transaction_id) VALUES
           (2, '2026-07-01 11:20:00', 'Paid to SHOP', 500, 'debit', 'T1', '512345678901', 'XXXXXX1111', 1),
           (2, '2026-07-03 10:00:00', 'Paid to LANDLORD', 900, 'debit', 'T3', 'U3', 'ICICI R', 2);",
    )
    .execute(&db.pool)
    .await
    .unwrap();
}

/// Two more Transactions and records of one Instrument that landed on both Accounts.
pub async fn seed_ambiguous_instrument(db: &TestDb) {
    sqlx::raw_sql(
        "INSERT INTO finance.transactions (account_id, statement_id, date, description, amount_paise, direction)
           VALUES (1, 1, '2026-07-05', 'x', 901, 'debit'), (2, 1, '2026-07-05', 'y', 902, 'debit');
         INSERT INTO finance.payment_records
           (statement_id, occurred_at, description, amount_paise, direction, instrument, transaction_id)
         VALUES (2, '2026-07-05 10:00:00', 'Paid to X', 901, 'debit', 'XXXXXX9999', 4),
                (2, '2026-07-05 10:00:00', 'Paid to Y', 902, 'debit', 'XXXXXX9999', 5)",
    )
    .execute(&db.pool)
    .await
    .unwrap();
}
