//! The figures derived from a Transfer's debit and the credits linked so far. Nothing here is
//! stored: it is always worked out from the amounts.

/// What a Transfer adds up to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Figures {
    /// What the linked credits add up to.
    pub credited_paise: i64,
    /// The credits so far add up to less than the debit.
    pub partial: bool,
    /// What is still on its way (counts as Money out until it arrives).
    pub remaining_paise: i64,
    /// What the credits add up to beyond the debit (counts as Money in).
    pub extra_paise: i64,
}

/// The figures of a debit of `debit_paise` with credits adding up to `credited_paise`.
#[must_use]
pub const fn figures(debit_paise: i64, credited_paise: i64) -> Figures {
    let gap = debit_paise - credited_paise;
    Figures {
        credited_paise,
        partial: gap > 0,
        remaining_paise: if gap > 0 { gap } else { 0 },
        extra_paise: if gap < 0 { -gap } else { 0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credits_short_of_the_debit_leave_a_remainder() {
        let f = figures(10_000, 4_000);
        assert_eq!(
            (f.partial, f.remaining_paise, f.extra_paise),
            (true, 6_000, 0)
        );
    }

    #[test]
    fn credits_equal_to_the_debit_complete_it() {
        let f = figures(10_000, 10_000);
        assert_eq!((f.partial, f.remaining_paise, f.extra_paise), (false, 0, 0));
    }

    #[test]
    fn credits_above_the_debit_are_extra() {
        let f = figures(10_000, 10_450);
        assert_eq!(
            (f.partial, f.remaining_paise, f.extra_paise),
            (false, 0, 450)
        );
        assert_eq!(f.credited_paise, 10_450);
    }
}
