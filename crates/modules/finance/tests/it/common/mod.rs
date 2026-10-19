//! Fixtures: a stage database with the Finance schema and small row builders.
#![allow(dead_code)]

pub mod app;
pub mod app_files;
pub mod cards;
pub mod instruments;
pub mod old_shape;
pub mod pairs;
pub mod results;
pub mod transfers;

use chrono::NaiveDate;
use vitarchy_core::test_db::TestDb;
use vitarchy_finance::reader::contract::{AccountKind, Direction, Transaction};
use vitarchy_finance::repo::{
    accounts::{self, AccountRow},
    statements::{self, NewStatement},
};

/// A stage database with the Finance migrations applied.
pub async fn finance_db(label: &str) -> TestDb {
    let db = TestDb::create(&format!("fin_{label}")).await;
    vitarchy_finance::module()
        .migrator()
        .run(&db.pool)
        .await
        .unwrap();
    db
}

pub fn date(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 7, day).unwrap()
}

/// A debit of `amount_paise` on the given July 2026 day.
pub fn tx(day: u32, description: &str, amount_paise: i64) -> Transaction {
    Transaction {
        date: date(day),
        time: None,
        description: description.to_owned(),
        amount_paise,
        direction: Direction::Debit,
        balance_paise: None,
        foreign_amount: None,
        foreign_currency: None,
        reference: None,
        source_id: None,
    }
}

pub async fn account(db: &TestDb, mask: &str) -> AccountRow {
    accounts::insert(
        &db.pool,
        "ICICI Bank",
        AccountKind::CreditCard,
        Some(mask),
        "Card",
    )
    .await
    .unwrap()
}

/// An unconfirmed, successfully read Statement; returns its id.
pub async fn statement(db: &TestDb, sha_char: char) -> i64 {
    let sha = sha_char.to_string().repeat(64);
    statements::insert(
        &db.pool,
        &NewStatement {
            file_sha256: &sha,
            file_name: "a.pdf",
            format: Some("pdf"),
            kind: Some("statement"),
            outcome: "read",
            result: Some(&serde_json::json!({})),
            reader_version: Some("1"),
        },
    )
    .await
    .unwrap()
}
