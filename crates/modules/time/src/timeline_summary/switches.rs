//! Switches: moving from one app to another between touching Active pieces.

use super::clip::Piece;

/// How many Switches `pieces` (in time order) hold. Two pieces make one only when both are
/// Active, they touch (the first ends where the second starts) and their apps differ. A new
/// window in the same app, Away, unknown Away or a gap means no Switch.
#[must_use]
pub fn count(pieces: &[Piece]) -> i64 {
    let count = pieces
        .windows(2)
        .filter(|w| {
            w[0].is_active()
                && w[1].is_active()
                && w[0].end_at == w[1].start_at
                && w[0].app != w[1].app
        })
        .count();
    i64::try_from(count).unwrap_or(i64::MAX)
}

#[allow(clippy::cast_precision_loss)] // counts and seconds are far below 2^52
/// Switches per Active hour; `None` with no Active time.
#[must_use]
pub fn per_active_hour(switches: i64, active_secs: i64) -> Option<f64> {
    (active_secs > 0).then(|| switches as f64 * 3600.0 / active_secs as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::model::Away;
    use crate::timeline_summary::test_support::{active_s, piece_s};

    #[test]
    fn a_different_app_right_after_is_a_switch() {
        let pieces = [
            active_s("Code", 0, 60),
            active_s("Mail", 60, 120),
            active_s("Code", 120, 180),
        ];
        assert_eq!(count(&pieces), 2);
    }

    #[test]
    fn a_new_window_in_the_same_app_is_not_a_switch() {
        assert_eq!(
            count(&[active_s("Code", 0, 60), active_s("Code", 60, 120)]),
            0
        );
    }

    #[test]
    fn a_gap_away_or_unknown_between_apps_is_not_a_switch() {
        let gap = [active_s("Code", 0, 60), active_s("Mail", 61, 120)];
        assert_eq!(count(&gap), 0);
        for between in [Away::Yes, Away::Unknown] {
            let pieces = [
                active_s("Code", 0, 60),
                piece_s("Lock", 60, 90, between),
                active_s("Mail", 90, 120),
            ];
            assert_eq!(count(&pieces), 0);
        }
    }

    #[test]
    fn rate_is_per_active_hour_and_absent_without_active_time() {
        assert_eq!(per_active_hour(6, 7200), Some(3.0));
        assert_eq!(per_active_hour(0, 0), None);
    }
}
