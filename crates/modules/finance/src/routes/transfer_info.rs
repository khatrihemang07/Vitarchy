//! How a Transfer is shown: on a Transaction's row, and as the answer to linking one.

use serde::Serialize;
use utoipa::ToSchema;

use crate::transfers::{Parts, Rung};

/// The Transfer a Transaction is part of, as its row shows it.
#[derive(Serialize, ToSchema)]
pub struct TransferInfo {
    /// Transfer id (`DELETE /transfers/{id}` unlinks it).
    pub id: i64,
    /// What it was linked on; `manual` when the person linked it.
    pub rung: Rung,
    /// The Accounts on the other side of this row.
    pub other_accounts: Vec<i64>,
    /// The credits so far add up to less than the debit.
    pub partial: bool,
    /// What is still on its way, in paise; it counts as Money out until it arrives.
    pub remaining_paise: i64,
    /// What the credits add up to beyond the debit, in paise; it counts as Money in.
    pub extra_paise: i64,
}

impl TransferInfo {
    /// The Transfer as seen from a row on `viewing_account_id`.
    #[must_use]
    pub fn of(parts: &Parts, viewing_account_id: i64) -> Self {
        let figures = parts.figures();
        Self {
            id: parts.id,
            rung: parts.rung,
            other_accounts: parts.other_accounts(viewing_account_id),
            partial: figures.partial,
            remaining_paise: figures.remaining_paise,
            extra_paise: figures.extra_paise,
        }
    }
}
