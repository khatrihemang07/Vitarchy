//! What a Read again would change, computed against what is stored. Nothing is ever deleted:
//! rows the engine no longer finds are only marked (ADR 0013).

use std::collections::HashSet;

use sqlx::PgConnection;

use super::printing_writes::{Target, plan_by_target};
use crate::{
    dedupe::{PrintingToAdd, read_again_diff},
    error::FinanceError,
    printings::Plan,
    problem::Refusal,
    reader::{ReaderResult, contract::Kind},
    record_key,
    repo::{payment_records, printings, statement_accounts, transactions},
};

/// The changes to one Account section.
#[derive(Debug)]
pub struct SectionDiff {
    /// The section's printed order.
    pub position: i32,
    /// The Account it was confirmed to.
    pub account_id: i64,
    /// Indexes of incoming rows that are new Transactions.
    pub add: Vec<usize>,
    /// Stored rows this Statement printed and the engine still finds.
    pub unchanged: Vec<i64>,
    /// Incoming rows that are stored Transactions this Statement never printed: each would
    /// gain a Printing.
    pub new_printings: Vec<PrintingToAdd>,
    /// Indexes of the incoming rows among those whose Printing would change what the
    /// Transaction shows.
    pub gaining: Vec<usize>,
    /// Transactions whose live Printing from this Statement the engine no longer finds.
    pub gone_transaction_ids: Vec<i64>,
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
    /// Whether applying it would add rows or Printings, or mark any as no longer found.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        match self {
            Self::Sections(sections) => sections.iter().any(|s| {
                !s.add.is_empty()
                    || !s.new_printings.is_empty()
                    || !s.gone_transaction_ids.is_empty()
            }),
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
        let here = printings::marks_of_statement(&mut *db, account_id, statement_id).await?;
        let diff = read_again_diff(&statement.transactions, &stored, &here);
        let targets: Vec<Target> = diff
            .new_printings
            .iter()
            .map(|p| Target {
                transaction_id: p.transaction_id,
                row: &statement.transactions[p.index],
            })
            .collect();
        let plans = plan_by_target(db, statement_id, &targets).await?;
        let mut new_printings = Vec::new();
        let mut gaining = Vec::new();
        let mut skipped_identical = 0;
        for (p, plan) in diff.new_printings.into_iter().zip(plans) {
            match plan {
                Plan::Add { gains } => {
                    gaining.extend(gains.then_some(p.index));
                    new_printings.push(p);
                }
                Plan::IdenticalKept => skipped_identical += 1,
                Plan::ThisStatement => {}
            }
        }
        tracing::debug!(
            statement_id,
            skipped_identical,
            "read again printings planned"
        );
        sections.push(SectionDiff {
            position: slot.position,
            account_id,
            add: diff.add,
            unchanged: diff.unchanged,
            new_printings,
            gaining,
            gone_transaction_ids: diff.gone_transaction_ids,
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
