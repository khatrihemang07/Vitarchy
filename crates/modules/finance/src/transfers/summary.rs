//! What one Transfer looks like from one of its Transactions' rows.

use super::{Figures, Rung, figures};

/// A stored Transfer's figures, as loaded for display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parts {
    /// Transfer id.
    pub id: i64,
    /// What it was linked on.
    pub rung: Rung,
    /// The debit's Account.
    pub debit_account_id: i64,
    /// The debit's amount.
    pub debit_paise: i64,
    /// The Account of each linked credit, in link order.
    pub credit_account_ids: Vec<i64>,
    /// What the credits add up to.
    pub credited_paise: i64,
}

impl Parts {
    /// The derived figures: credited sum, partial, remainder and extra.
    #[must_use]
    pub const fn figures(&self) -> Figures {
        figures(self.debit_paise, self.credited_paise)
    }

    /// The Accounts on the other side of the row on `viewing_account_id`: the credits'
    /// Accounts (each once) for the debit, the debit's Account for a credit.
    #[must_use]
    pub fn other_accounts(&self, viewing_account_id: i64) -> Vec<i64> {
        if viewing_account_id != self.debit_account_id {
            return vec![self.debit_account_id];
        }
        let mut accounts = Vec::new();
        for id in &self.credit_account_ids {
            if !accounts.contains(id) {
                accounts.push(*id);
            }
        }
        accounts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split() -> Parts {
        Parts {
            id: 7,
            rung: Rung::BillPayment,
            debit_account_id: 1,
            debit_paise: 10_000,
            credit_account_ids: vec![2, 3, 2],
            credited_paise: 6_000,
        }
    }

    #[test]
    fn the_debit_row_sees_each_receiving_account_once() {
        assert_eq!(split().other_accounts(1), vec![2, 3]);
    }

    #[test]
    fn a_credit_row_sees_the_paying_account() {
        assert_eq!(split().other_accounts(3), vec![1]);
    }

    #[test]
    fn figures_come_from_the_amounts() {
        let f = split().figures();
        assert_eq!(
            (f.partial, f.remaining_paise, f.extra_paise),
            (true, 4_000, 0)
        );
    }
}
