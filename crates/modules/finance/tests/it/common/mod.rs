//! Fixtures: a stage database with the Finance schema and small row builders.
#![allow(dead_code)]

pub mod app;
pub mod cards;
pub mod results;

use chrono::NaiveDate;
use vitarchy_core::test_db::TestDb;
use vitarchy_finance::reader::contract::{AccountKind, Direction, PaymentRecord, Transaction};
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
        description: description.to_owned(),
        amount_paise,
        direction: Direction::Debit,
        balance_paise: None,
        foreign_amount: None,
        foreign_currency: None,
        reference: None,
        addon_card: None,
        source_id: None,
    }
}

pub fn payment(source_id: Option<&str>, utr: Option<&str>, amount_paise: i64) -> PaymentRecord {
    PaymentRecord {
        occurred_at: date(5).and_hms_opt(11, 20, 0).unwrap(),
        description: "Paid to SOME MERCHANT".to_owned(),
        amount_paise,
        direction: Direction::Debit,
        source_id: source_id.map(str::to_owned),
        utr: utr.map(str::to_owned),
        instrument: Some("XXXXXX4321".to_owned()),
        instrument_issuer: None,
        instrument_digits: Some("4321".to_owned()),
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
