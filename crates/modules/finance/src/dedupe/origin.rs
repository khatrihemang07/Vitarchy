//! Where a Printing comes from: a bank or card issuer's Statement, or a payment app's
//! history. Only the unique-amount-and-date rung cares, because it may join a payment app's
//! row to an issuer's row and never two of the same kind.

use crate::reader::contract::Kind;

/// What kind of file printed a row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// A bank or card issuer's Statement.
    Statement,
    /// A payment app's history.
    PaymentHistory,
}

impl From<Kind> for Origin {
    fn from(kind: Kind) -> Self {
        match kind {
            Kind::Statement => Self::Statement,
            Kind::PaymentHistory => Self::PaymentHistory,
        }
    }
}

/// The origins of the live Printings a row stands for. An incoming row has one; a stored
/// Transaction has one for each kind of file that printed it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Origins {
    /// Origin unknown: the unique-amount-and-date rung never applies.
    #[default]
    Unknown,
    /// Printed by exactly this one kind of file.
    Only(Origin),
    /// Printed by both kinds of file.
    Both,
}

impl Origins {
    /// Printed by exactly this one kind of file.
    #[must_use]
    pub const fn of(origin: Origin) -> Self {
        Self::Only(origin)
    }

    /// Whether ids printed by these two kinds of file live in one namespace. A bank's row
    /// number and a payment app's transaction id do not, so they never agree or disagree; a
    /// Transaction both printed has ids of both, which its Printings tell apart.
    #[must_use]
    pub fn share_ids(self, other: Self) -> bool {
        self != Self::Both && self == other
    }

    /// Whether one side is only a payment app's and the other only an issuer's.
    #[must_use]
    pub fn complement(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Only(a), Self::Only(b)) if a != b
        )
    }
}
