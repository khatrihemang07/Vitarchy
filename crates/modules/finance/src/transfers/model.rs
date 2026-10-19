//! The facts detection works from and what it answers with. Plain data: the database side
//! fills it in and writes the answer back.

use std::collections::HashSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::reader::contract::{AccountKind, Direction};

/// How much more than the debit a bill payment's credits may add up to (₹50), from cashback
/// and the like. No other kind of Transfer has any tolerance.
pub const BILL_TOLERANCE_PAISE: i64 = 5_000;

/// The evidence a Transfer was linked on, strongest first; `Manual` is the person's own link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Rung {
    /// The debit's reference appears in the credit.
    Reference,
    /// The debit names the other Account's number.
    AccountNumber,
    /// A card-bill payment and the receipts it paid.
    BillPayment,
    /// The only debit and credit of that amount around that day.
    UniqueAmount,
    /// Linked by the person.
    Manual,
}

impl Rung {
    /// Every rung, strongest first; `from_id` and `TryFrom` derive from this one list.
    pub const ALL: [Self; 5] = [
        Self::Reference,
        Self::AccountNumber,
        Self::BillPayment,
        Self::UniqueAmount,
        Self::Manual,
    ];

    /// The rung stored as `id`, if any.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|rung| rung.id() == id)
    }

    /// The spelling stored in the database.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::AccountNumber => "account_number",
            Self::BillPayment => "bill_payment",
            Self::UniqueAmount => "unique_amount",
            Self::Manual => "manual",
        }
    }
}

impl TryFrom<String> for Rung {
    type Error = String;

    fn try_from(id: String) -> Result<Self, String> {
        Self::from_id(&id).ok_or_else(|| format!("unknown Rung: {id}"))
    }
}

/// An own, non-hidden Account as detection sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountInfo {
    /// Its id.
    pub id: i64,
    /// Savings, card or loan.
    pub kind: AccountKind,
    /// Every run of trailing digits it is known by (its mask and remembered identifiers).
    pub digits: Vec<String>,
}

/// A Transaction as detection sees it: its resolved view plus what every Printing printed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    /// Transaction id.
    pub id: i64,
    /// Its Account.
    pub account_id: i64,
    /// Money out (debit) or Money in (credit).
    pub direction: Direction,
    /// Amount, above 0.
    pub amount_paise: i64,
    /// The printed date.
    pub date: NaiveDate,
    /// The resolved description and every Printing's description.
    pub texts: Vec<String>,
    /// The resolved reference and every Printing's reference.
    pub references: Vec<String>,
}

/// A Transfer that is already stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistingTransfer {
    /// The Money-out Transaction.
    pub debit_id: i64,
    /// The Money-in Transactions linked so far.
    pub credit_ids: Vec<i64>,
    /// What they add up to.
    pub linked_paise: i64,
    /// What it was linked on.
    pub rung: Rung,
}

/// Everything one detection run looks at.
#[derive(Debug, Clone, Copy)]
pub struct Input<'a> {
    /// The own, non-hidden Accounts; Transactions of any other Account are ignored.
    pub accounts: &'a [AccountInfo],
    /// The Transactions to pair, linked ones included.
    pub candidates: &'a [Candidate],
    /// The Transfers already stored.
    pub transfers: &'a [ExistingTransfer],
    /// Pairs (debit, credit) the person unlinked.
    pub rejected: &'a HashSet<(i64, i64)>,
    /// When set, only Transfers whose debit is dated inside it are reported; the rest of the
    /// candidates are there so uniqueness is judged against everything nearby.
    pub focus: Option<(NaiveDate, NaiveDate)>,
}

/// One new link: credits to add to the Transfer of `debit_id` (made if it has none).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    /// The Money-out Transaction.
    pub debit_id: i64,
    /// The Money-in Transactions to link to it.
    pub credit_ids: Vec<i64>,
    /// What it was found on.
    pub rung: Rung,
}

/// How many links each automatic rung found.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RungCounts {
    /// Rung 1.
    pub reference: usize,
    /// Rung 2.
    pub account_number: usize,
    /// Rung 3.
    pub bill_payment: usize,
    /// Rung 4.
    pub unique_amount: usize,
}

impl RungCounts {
    /// Counts one link of `rung`.
    pub fn add(&mut self, rung: Rung) {
        match rung {
            Rung::Reference => self.reference += 1,
            Rung::AccountNumber => self.account_number += 1,
            Rung::BillPayment => self.bill_payment += 1,
            Rung::UniqueAmount => self.unique_amount += 1,
            Rung::Manual => {}
        }
    }
}

/// What a detection run found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Detection {
    /// The new links, strongest rung first.
    pub links: Vec<Link>,
    /// How many links each rung found.
    pub found: RungCounts,
    /// Debits or credits that fit more than one partner and were left alone.
    pub ambiguous: usize,
}

#[cfg(test)]
mod tests {
    use super::Rung;

    #[test]
    fn every_rung_round_trips_through_its_stored_spelling() {
        for rung in Rung::ALL {
            assert_eq!(Rung::from_id(rung.id()), Some(rung));
            assert_eq!(Rung::try_from(rung.id().to_owned()), Ok(rung));
        }
        assert_eq!(Rung::from_id("guess"), None);
    }
}
