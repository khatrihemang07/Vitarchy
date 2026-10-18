//! One section of a preview: the proposal, its Account match and the new-versus-already-stored count.

use std::collections::HashMap;

use super::{
    model::{AccountRef, CheckView, PeriodView, ProposedAccountView, SectionPreview},
    rows,
};
use crate::{
    accounts::{AccountIdentity, Match, find_for},
    dedupe::{Origin, StoredEvidence},
    printings::Printing,
    reader::contract::{Direction, Kind, Statement},
    repo::statement_accounts::StatementAccountRow,
};

/// How a section finds its Account: by the kind of file it is in, the ways Accounts are known
/// and the instruments the person has confirmed.
pub struct Matcher<'a> {
    /// What the file is.
    pub kind: Option<Kind>,
    /// Every way an Account is known.
    pub known: &'a [AccountIdentity],
    /// The Account each remembered instrument is.
    pub instruments: &'a HashMap<String, i64>,
}

/// What the database knows that a section needs: Account names, how Accounts are matched,
/// the rows stored per Account and their Printings.
pub struct Lookup<'a> {
    /// Account names by id.
    pub names: &'a HashMap<i64, String>,
    /// How a section finds its Account.
    pub matcher: Matcher<'a>,
    /// Each Account's stored rows as matching sees them, by Account id.
    pub stored: &'a HashMap<i64, Vec<StoredEvidence>>,
    /// The Printings of every stored row, by Transaction id.
    pub printings: &'a HashMap<i64, Vec<Printing>>,
    /// The Statement being previewed.
    pub statement_id: i64,
}

impl Matcher<'_> {
    /// The Account a section resolves to: the one stored on its slot, else a live match.
    pub fn resolve(&self, slot: &StatementAccountRow, statement: &Statement) -> Match {
        if let Some(id) = slot.account_id {
            return Match::One(id);
        }
        find_for(self.kind, &statement.account, self.instruments, self.known)
    }
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
    let found = lookup.matcher.resolve(slot, statement);
    let matched_without_number = matches!(found, Match::OneWithoutNumber(_));
    let matched = found.single();
    let candidates = match found {
        Match::Several(ids) => ids,
        _ => vec![],
    };
    let stored = matched
        .and_then(|id| lookup.stored.get(&id))
        .map_or(&[][..], Vec::as_slice);
    let rows = rows::build(
        lookup.statement_id,
        lookup.matcher.kind.map_or(Origin::Statement, Origin::from),
        &statement.transactions,
        stored,
        lookup.printings,
    );
    let transaction_count = rows.views.len();
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
        transaction_count,
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
        new_count: rows.new_count,
        already_stored_count: transaction_count - rows.new_count,
        enriched_count: rows.enriched_count,
        transactions: rows.views,
    }
}
