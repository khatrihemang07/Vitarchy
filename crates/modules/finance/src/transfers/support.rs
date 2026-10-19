//! Builders for the unit tests of detection. Every value is invented.

use std::collections::HashSet;

use chrono::NaiveDate;

use super::{AccountInfo, Candidate, Detection, ExistingTransfer, Input, Link, Rung, detect};
use crate::reader::contract::{AccountKind, Direction};

pub fn account(id: i64, kind: AccountKind, digits: &str) -> AccountInfo {
    AccountInfo {
        id,
        kind,
        digits: vec![digits.to_owned()],
    }
}

pub fn savings(id: i64, digits: &str) -> AccountInfo {
    account(id, AccountKind::Savings, digits)
}

pub fn card(id: i64, digits: &str) -> AccountInfo {
    account(id, AccountKind::CreditCard, digits)
}

pub fn loan(id: i64, digits: &str) -> AccountInfo {
    account(id, AccountKind::Loan, digits)
}

pub fn day(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 7, day).expect("valid date")
}

fn candidate(
    id: i64,
    account_id: i64,
    direction: Direction,
    on: u32,
    paise: i64,
    text: &str,
) -> Candidate {
    Candidate {
        id,
        account_id,
        direction,
        amount_paise: paise,
        date: day(on),
        texts: vec![text.to_owned()],
        references: Vec::new(),
    }
}

pub fn debit(id: i64, account_id: i64, on: u32, paise: i64, text: &str) -> Candidate {
    candidate(id, account_id, Direction::Debit, on, paise, text)
}

pub fn credit(id: i64, account_id: i64, on: u32, paise: i64, text: &str) -> Candidate {
    candidate(id, account_id, Direction::Credit, on, paise, text)
}

pub fn referenced(mut c: Candidate, reference: &str) -> Candidate {
    c.references.push(reference.to_owned());
    c
}

pub fn also_printed(mut c: Candidate, text: &str) -> Candidate {
    c.texts.push(text.to_owned());
    c
}

/// Detection over `candidates` with nothing linked or rejected yet.
pub fn run(accounts: &[AccountInfo], candidates: &[Candidate]) -> Detection {
    run_with(accounts, candidates, &[], &HashSet::new())
}

pub fn run_with(
    accounts: &[AccountInfo],
    candidates: &[Candidate],
    transfers: &[ExistingTransfer],
    rejected: &HashSet<(i64, i64)>,
) -> Detection {
    detect(&Input {
        accounts,
        candidates,
        transfers,
        rejected,
        focus: None,
    })
}

pub fn link(debit_id: i64, credit_ids: &[i64], rung: Rung) -> Link {
    Link {
        debit_id,
        credit_ids: credit_ids.to_vec(),
        rung,
    }
}

/// A stored Transfer of `debit_id` with credits `credit_ids` adding up to `linked_paise`.
pub fn stored(
    debit_id: i64,
    credit_ids: &[i64],
    linked_paise: i64,
    rung: Rung,
) -> ExistingTransfer {
    ExistingTransfer {
        debit_id,
        credit_ids: credit_ids.to_vec(),
        linked_paise,
        rung,
    }
}
