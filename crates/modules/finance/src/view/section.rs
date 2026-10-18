//! One section of a preview: the proposal, its Account match and the new-versus-duplicate count.

use std::collections::HashMap;

use super::{
    model::{AccountRef, CheckView, PeriodView, ProposedAccountView, SectionPreview},
    transaction::TransactionView,
};
use crate::{
    accounts::{AccountIdentity, Match},
    dedupe::{Evidence, new_rows},
    match_account,
    reader::contract::{Direction, Statement},
    repo::statement_accounts::StatementAccountRow,
};

/// What the database knows that a section needs: Account names, the ways Accounts are known
/// and the evidence of the rows stored per Account.
pub struct Lookup<'a> {
    /// Account names by id.
    pub names: &'a HashMap<i64, String>,
    /// Every way an Account is known.
    pub known: &'a [AccountIdentity],
    /// The evidence of each Account's stored rows, by Account id.
    pub stored: &'a HashMap<i64, Vec<Evidence>>,
}

/// The Account a section resolves to: the one stored on its slot, else a live match.
pub fn resolve(
    slot: &StatementAccountRow,
    statement: &Statement,
    known: &[AccountIdentity],
) -> Match {
    if let Some(id) = slot.account_id {
        return Match::One(id);
    }
    let proposal = &statement.account;
    match_account(
        proposal.issuer.as_deref(),
        proposal.kind,
        proposal.mask.as_deref(),
        known,
    )
}

fn account_ref(lookup: &Lookup<'_>, id: i64) -> Option<AccountRef> {
    Some(AccountRef {
        id,
        name: lookup.names.get(&id)?.clone(),
    })
}

fn total(statement: &Statement, direction: Direction) -> i64 {
    statement
        .transactions
        .iter()
        .filter(|tx| tx.direction == direction)
        .map(|tx| tx.amount_paise)
        .sum()
}

/// Builds the preview of one section.
pub fn build(
    slot: &StatementAccountRow,
    statement: &Statement,
    lookup: &Lookup<'_>,
) -> SectionPreview {
    let found = resolve(slot, statement, lookup.known);
    let matched_without_number = matches!(found, Match::OneWithoutNumber(_));
    let matched = found.single();
    let candidates = match found {
        Match::Several(ids) => ids,
        _ => vec![],
    };
    let stored = matched
        .and_then(|id| lookup.stored.get(&id))
        .map_or(&[][..], Vec::as_slice);
    let fresh = new_rows(&statement.transactions, stored);
    let transactions: Vec<TransactionView> = statement
        .transactions
        .iter()
        .enumerate()
        .map(|(index, tx)| TransactionView::of(tx, !fresh.contains(&index)))
        .collect();
    let new_count = fresh.len();
    SectionPreview {
        position: slot.position,
        proposed: ProposedAccountView {
            issuer: statement.account.issuer.clone(),
            kind: statement.account.kind.map(|k| k.id().to_owned()),
            mask: statement.account.mask.clone(),
        },
        matched_account: matched.and_then(|id| account_ref(lookup, id)),
        matched_without_number,
        candidate_accounts: candidates
            .into_iter()
            .filter_map(|id| account_ref(lookup, id))
            .collect(),
        period: PeriodView {
            start: statement.period.start,
            end: statement.period.end,
        },
        transaction_count: transactions.len(),
        total_debit_paise: total(statement, Direction::Debit),
        total_credit_paise: total(statement, Direction::Credit),
        checks: statement
            .checks
            .iter()
            .map(|c| CheckView {
                name: c.name.clone(),
                outcome: c.outcome.id().to_owned(),
                detail: c.detail.clone(),
            })
            .collect(),
        new_count,
        duplicate_count: transactions.len() - new_count,
        transactions,
    }
}
