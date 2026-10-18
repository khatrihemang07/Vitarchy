//! The headline figures of a set of pieces, one function so Period and Usual agree.

use super::{
    buckets::{self, Seconds},
    clip::Piece,
    focus, switches,
};

/// What a stretch of Timeline amounts to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Figures {
    /// Seconds by Away.
    pub seconds: Seconds,
    /// Switches between apps.
    pub switches: i64,
    /// Focus stretch figures.
    pub focus: focus::Focus,
}

/// All figures over `pieces` (in time order).
#[must_use]
pub fn measure(pieces: &[Piece]) -> Figures {
    Figures {
        seconds: buckets::seconds(pieces),
        switches: switches::count(pieces),
        focus: focus::measure(pieces),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline_summary::test_support::active_s;

    #[test]
    fn measure_combines_seconds_switches_and_focus() {
        let pieces = [active_s("Code", 0, 1800), active_s("Mail", 1800, 1900)];
        let figures = measure(&pieces);
        assert_eq!(
            (
                figures.seconds.active,
                figures.switches,
                figures.focus.longest_secs
            ),
            (1900, 1, 1800)
        );
    }
}
