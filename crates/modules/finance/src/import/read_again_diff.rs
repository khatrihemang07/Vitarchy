//! What a Read again would change, computed against what is stored. Nothing is ever deleted:
//! rows the engine no longer finds are only marked (ADR 0013).

use std::collections::HashSet;

use sqlx::PgConnection;

use crate::{
    dedupe::read_again_diff,
    error::FinanceError,
    problem::Refusal,
    reader::{ReaderResult, contract::Kind},
    record_key,
    repo::{payment_records, statement_accounts, transactions, transactions_read_again},
};

/// The changes to one Account section.
#[derive(Debug)]
pub struct SectionDiff {
    /// The section's printed order.
    pub position: i32,
    /// The Account it was confirmed to.
    pub account_id: i64,
    /// Indexes of incoming rows that are new.
    pub add: Vec<usize>,
    /// Stored rows still found.
    pub unchanged: Vec<i64>,
    /// This Statement's stored rows the engine no longer finds.
    pub no_longer_found: Vec<i64>,
}

/// The changes to a whole file.
#[derive(Debug)]
pub enum Diff {
    /// A Statement: one diff per Account section.
    Sections(Vec<SectionDiff>),
    /// A Payment history: the new records.
    Records {
        /// Indexes of incoming records that are new.
        add: Vec<usize>,
        /// Incoming records already stored.
        unchanged: usize,
    },
}

impl Diff {
    /// Whether applying it would add rows or mark any as no longer found.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        match self {
            Self::Sections(sections) => sections
                .iter()
                .any(|s| !s.add.is_empty() || !s.no_longer_found.is_empty()),
            Self::Records { add, .. } => !add.is_empty(),
        }
    }
}

/// Compares `result` with what Statement `statement_id` stored.
///
/// # Errors
/// The file now lists different Accounts than were confirmed, or the database failed.
pub async fn compute(
    db: &mut PgConnection,
    statement_id: i64,
    result: &ReaderResult,
) -> Result<Diff, FinanceError> {
    if result.kind == Some(Kind::PaymentHistory) {
        return Ok(records(db, result).await?);
    }
    let slots = statement_accounts::list(&mut *db, statement_id).await?;
    if slots.len() != result.statements.len() {
        return Err(Refusal::SECTIONS_CHANGED.into());
    }
    let mut sections = Vec::new();
    for (slot, statement) in slots.iter().zip(&result.statements) {
        let account_id = slot.account_id.ok_or(Refusal::SECTIONS_CHANGED)?;
        let stored = transactions::stored_keys(&mut *db, account_id).await?;
        let diff = read_again_diff(&statement.transactions, &stored);
        let mine: HashSet<i64> =
            transactions_read_again::ids_of_statement(&mut *db, account_id, statement_id)
                .await?
                .into_iter()
                .collect();
        sections.push(SectionDiff {
            position: slot.position,
            account_id,
            add: diff.add,
            unchanged: diff.unchanged,
            no_longer_found: diff
                .no_longer_found
                .into_iter()
                .filter(|id| mine.contains(id))
                .collect(),
        });
    }
    Ok(Diff::Sections(sections))
}

async fn records(db: &mut PgConnection, result: &ReaderResult) -> Result<Diff, sqlx::Error> {
    let stored: HashSet<String> = payment_records::all(&mut *db)
        .await?
        .iter()
        .map(record_key::of_stored)
        .collect();
    let add: Vec<usize> = result
        .payment_records
        .iter()
        .enumerate()
        .filter(|(_, record)| !stored.contains(&record_key::of_incoming(record)))
        .map(|(index, _)| index)
        .collect();
    Ok(Diff::Records {
        unchanged: result.payment_records.len() - add.len(),
        add,
    })
}
