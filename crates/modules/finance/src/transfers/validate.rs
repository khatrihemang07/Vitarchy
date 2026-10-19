//! Whether a Transfer the person links by hand makes sense. Unlike detection it may be
//! partial. Its credits may exceed the debit only by the bill-payment tolerance, and only when
//! every credit is on a credit card (ADR 0017); otherwise they add up to at most the debit.

use std::collections::HashSet;

use super::{BILL_TOLERANCE_PAISE, Candidate};
use crate::reader::contract::Direction;

/// Why a hand-made Transfer is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Invalid {
    /// No credit was given.
    NoCredits,
    /// The first Transaction is not Money out.
    NotMoneyOut,
    /// A credit is not Money in.
    NotMoneyIn,
    /// A credit is on the debit's own Account.
    SameAccount,
    /// The same credit was given twice.
    DuplicateCredit,
    /// The credits add up to more than the debit allows.
    TooMuch,
}

/// How much more than the debit the credits may add up to: the bill-payment tolerance when
/// every credit is on a credit card, nothing otherwise. The one place this rule is stated.
#[must_use]
pub const fn extra_allowed(all_credits_on_cards: bool) -> i64 {
    if all_credits_on_cards {
        BILL_TOLERANCE_PAISE
    } else {
        0
    }
}

/// Checks a debit and credits for a Transfer, `all_credits_on_cards` saying whether every
/// credit is on a credit-card Account. Whether the Accounts are hidden or the Transactions
/// already linked is for the caller, who has the database.
///
/// # Errors
/// The first rule the Transfer breaks.
pub fn validate_manual(
    debit: &Candidate,
    credits: &[Candidate],
    all_credits_on_cards: bool,
) -> Result<(), Invalid> {
    if credits.is_empty() {
        return Err(Invalid::NoCredits);
    }
    if debit.direction != Direction::Debit {
        return Err(Invalid::NotMoneyOut);
    }
    let mut seen = HashSet::new();
    for credit in credits {
        if credit.direction != Direction::Credit {
            return Err(Invalid::NotMoneyIn);
        }
        if credit.account_id == debit.account_id {
            return Err(Invalid::SameAccount);
        }
        if !seen.insert(credit.id) {
            return Err(Invalid::DuplicateCredit);
        }
    }
    let total: i64 = credits.iter().map(|c| c.amount_paise).sum();
    if total > debit.amount_paise + extra_allowed(all_credits_on_cards) {
        return Err(Invalid::TooMuch);
    }
    Ok(())
}

/// Whether a credit of `credit_paise` could be (part of) the Transfer of a debit of
/// `debit_paise`: a partial Transfer is fine, one with more than the allowed extra is not.
#[must_use]
pub const fn fits(debit_paise: i64, credit_paise: i64, credit_on_card: bool) -> bool {
    credit_paise <= debit_paise + extra_allowed(credit_on_card)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transfers::support::{credit, debit};

    #[test]
    fn a_debit_and_smaller_credits_on_other_accounts_are_valid() {
        let credits = [credit(2, 2, 5, 400, "a"), credit(3, 3, 5, 500, "b")];
        assert_eq!(
            validate_manual(&debit(1, 1, 5, 1_000, "out"), &credits, false),
            Ok(())
        );
    }

    #[test]
    fn at_least_one_credit_is_needed() {
        assert_eq!(
            validate_manual(&debit(1, 1, 5, 1_000, "o"), &[], false),
            Err(Invalid::NoCredits)
        );
    }

    #[test]
    fn directions_must_be_opposite() {
        let credits = [credit(2, 2, 5, 400, "a")];
        assert_eq!(
            validate_manual(&credit(1, 1, 5, 400, "x"), &credits, false),
            Err(Invalid::NotMoneyOut)
        );
        let wrong = [debit(2, 2, 5, 400, "a")];
        assert_eq!(
            validate_manual(&debit(1, 1, 5, 400, "x"), &wrong, false),
            Err(Invalid::NotMoneyIn)
        );
    }

    #[test]
    fn a_credit_on_the_debits_own_account_is_refused() {
        let credits = [credit(2, 1, 5, 400, "a")];
        assert_eq!(
            validate_manual(&debit(1, 1, 5, 400, "x"), &credits, false),
            Err(Invalid::SameAccount)
        );
    }

    #[test]
    fn the_same_credit_twice_is_refused() {
        let credits = [credit(2, 2, 5, 100, "a"), credit(2, 2, 5, 100, "a")];
        assert_eq!(
            validate_manual(&debit(1, 1, 5, 400, "x"), &credits, false),
            Err(Invalid::DuplicateCredit)
        );
    }

    #[test]
    fn up_to_fifty_rupees_extra_is_allowed_only_when_every_credit_is_on_a_card() {
        let debit = debit(1, 1, 5, 10_000, "x");
        let at_limit = [credit(2, 2, 5, 15_000, "a")];
        assert_eq!(validate_manual(&debit, &at_limit, true), Ok(()));
        assert_eq!(
            validate_manual(&debit, &at_limit, false),
            Err(Invalid::TooMuch)
        );
        let over = [credit(2, 2, 5, 15_001, "a")];
        assert_eq!(validate_manual(&debit, &over, true), Err(Invalid::TooMuch));
    }

    #[test]
    fn without_cards_the_credits_may_only_add_up_to_the_debit() {
        let debit = debit(1, 1, 5, 10_000, "x");
        let exact = [credit(2, 2, 5, 6_000, "a"), credit(3, 3, 5, 4_000, "b")];
        assert_eq!(validate_manual(&debit, &exact, false), Ok(()));
        let one_over = [credit(2, 2, 5, 10_001, "a")];
        assert_eq!(
            validate_manual(&debit, &one_over, false),
            Err(Invalid::TooMuch)
        );
    }

    #[test]
    fn fits_uses_the_same_rule() {
        assert!(fits(10_000, 15_000, true));
        assert!(!fits(10_000, 15_001, true));
        assert!(fits(10_000, 10_000, false));
        assert!(!fits(10_000, 10_001, false));
        assert!(fits(10_000, 1, false));
    }
}
