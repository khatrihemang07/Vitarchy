//! Canned reader results for the route tests.

use vitarchy_finance::reader::contract::{
    AccountKind, Check, CheckOutcome, Diagnostics, Direction, Format, Kind, Outcome, PaymentRecord,
    Period, ProposedAccount, ReaderResult, Statement, Summary, Transaction,
};

use super::{date, tx};

fn diagnostics() -> Diagnostics {
    Diagnostics {
        stage: None,
        code: None,
        pages: Some(2),
        rows: None,
    }
}

/// A credit of `amount_paise` on the given July 2026 day.
pub fn credit(day: u32, description: &str, amount_paise: i64) -> Transaction {
    Transaction {
        direction: Direction::Credit,
        ..tx(day, description, amount_paise)
    }
}

/// A debit that also carries a balance after.
pub fn with_balance(day: u32, description: &str, amount_paise: i64, balance: i64) -> Transaction {
    Transaction {
        balance_paise: Some(balance),
        ..tx(day, description, amount_paise)
    }
}

/// An ICICI card section with the given rows and all Checks passing.
pub fn card_section(mask: &str, rows: Vec<Transaction>) -> Statement {
    section(
        Some("ICICI Bank"),
        Some(AccountKind::CreditCard),
        mask,
        rows,
    )
}

/// A section whose proposed Account may be incomplete.
pub fn section(
    issuer: Option<&str>,
    kind: Option<AccountKind>,
    mask: &str,
    rows: Vec<Transaction>,
) -> Statement {
    Statement {
        account: ProposedAccount {
            issuer: issuer.map(str::to_owned),
            kind,
            mask: Some(mask.to_owned()),
        },
        period: Period {
            start: Some(date(1)),
            end: Some(date(30)),
        },
        summary: Summary {
            opening_paise: None,
            closing_paise: None,
            total_debit_paise: None,
            total_credit_paise: None,
            debit_count: None,
            credit_count: None,
        },
        transactions: rows,
        checks: vec![Check {
            name: "balance_chain".to_owned(),
            outcome: CheckOutcome::Passed,
            detail: serde_json::json!({"rows": 1}),
        }],
    }
}

/// The same section as if the file printed no account number.
pub fn without_number(mut section: Statement) -> Statement {
    section.account.mask = None;
    section
}

/// The same section with one failed Check.
pub fn failing(mut section: Statement) -> Statement {
    section.checks[0].outcome = CheckOutcome::Failed;
    section
}

/// A read Statement file with these sections.
pub fn read(sections: Vec<Statement>) -> ReaderResult {
    ReaderResult {
        version: 1,
        outcome: Outcome::Read,
        format: Some(Format::Pdf),
        encrypted: false,
        unlocked_with: None,
        kind: Some(Kind::Statement),
        statements: sections,
        payment_records: vec![],
        diagnostics: diagnostics(),
    }
}

/// The same file, encrypted and opened by the first saved password.
pub fn unlocked(mut result: ReaderResult) -> ReaderResult {
    result.encrypted = true;
    result.unlocked_with = Some(0);
    result
}

/// A file no saved password opened.
pub fn locked() -> ReaderResult {
    ReaderResult {
        outcome: Outcome::Locked,
        encrypted: true,
        kind: None,
        diagnostics: Diagnostics {
            stage: Some("open".to_owned()),
            code: Some("no_password_worked".to_owned()),
            pages: None,
            rows: None,
        },
        ..read(vec![])
    }
}

/// A Payment history with these records.
pub fn history(records: Vec<PaymentRecord>) -> ReaderResult {
    ReaderResult {
        format: Some(Format::Csv),
        kind: Some(Kind::PaymentHistory),
        payment_records: records,
        ..read(vec![])
    }
}
