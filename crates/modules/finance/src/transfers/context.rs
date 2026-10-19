//! What every rung shares: the candidates split by direction, the Accounts they may use, and
//! which Transactions earlier rungs (or earlier runs) already linked.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;

use super::{AccountInfo, Candidate, ExistingTransfer, Input, Link, markers::Markers};
use crate::reader::contract::Direction;

/// The input of one run, indexed for the rungs.
pub struct Context<'a> {
    /// The own, non-hidden Accounts by id.
    pub accounts: HashMap<i64, &'a AccountInfo>,
    /// Money-out candidates on those Accounts.
    pub debits: Vec<&'a Candidate>,
    /// Money-in candidates on those Accounts.
    pub credits: Vec<&'a Candidate>,
    /// The stored Transfers by debit.
    pub transfers: HashMap<i64, &'a ExistingTransfer>,
    rejected: &'a HashSet<(i64, i64)>,
    /// The card-bill markers.
    pub markers: &'static Markers,
}

impl<'a> Context<'a> {
    /// Indexes `input`, dropping candidates of Accounts it does not list.
    #[must_use]
    pub fn new(input: &'a Input<'a>) -> Self {
        let accounts: HashMap<i64, &AccountInfo> =
            input.accounts.iter().map(|a| (a.id, a)).collect();
        let known = input
            .candidates
            .iter()
            .filter(|c| accounts.contains_key(&c.account_id));
        let (debits, credits) = known.partition(|c| c.direction == Direction::Debit);
        Self {
            accounts,
            debits,
            credits,
            transfers: input.transfers.iter().map(|t| (t.debit_id, t)).collect(),
            rejected: input.rejected,
            markers: Markers::built_in(),
        }
    }

    /// Whether `debit` and `credit` may be linked at all: different Accounts, and the person
    /// has not unlinked this pair.
    #[must_use]
    pub fn pairable(&self, debit: &Candidate, credit: &Candidate) -> bool {
        debit.account_id != credit.account_id && !self.rejected.contains(&(debit.id, credit.id))
    }
}

/// Transactions already in a Transfer, so no rung uses one twice.
#[derive(Default)]
pub struct Used {
    debits: HashSet<i64>,
    credits: HashSet<i64>,
}

impl Used {
    /// What the stored Transfers already use.
    #[must_use]
    pub fn of(transfers: &[ExistingTransfer]) -> Self {
        let mut used = Self::default();
        for transfer in transfers {
            used.debits.insert(transfer.debit_id);
            used.credits.extend(&transfer.credit_ids);
        }
        used
    }

    /// Marks a new link's Transactions as used.
    pub fn take(&mut self, link: &Link) {
        self.debits.insert(link.debit_id);
        self.credits.extend(&link.credit_ids);
    }

    /// Whether this debit has no Transfer yet.
    #[must_use]
    pub fn debit_free(&self, id: i64) -> bool {
        !self.debits.contains(&id)
    }

    /// Whether this credit is in no Transfer yet.
    #[must_use]
    pub fn credit_free(&self, id: i64) -> bool {
        !self.credits.contains(&id)
    }
}

/// Whole days between two dates, whichever is first.
#[must_use]
pub fn days_apart(a: NaiveDate, b: NaiveDate) -> i64 {
    (a - b).num_days().abs()
}
