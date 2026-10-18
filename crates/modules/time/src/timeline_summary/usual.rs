//! The Usual for Timeline figures: how the Days before a Period usually went.
//!
//! Same semantics as the Report's Usual (`report::usual`, `report::scaling`): per-Day average,
//! shortest and longest over the Days that have Timeline, how many those were, and totals
//! scaled to the Days that have passed against the Days of history available.

use std::collections::BTreeMap;

use chrono::NaiveDate;

use super::{
    clip::Piece,
    figures::{self, Figures},
    model::{UsualFigure, UsualTimeline},
};
use crate::report::scaling::{Scaling, Totals};
use crate::report::stats::spread;

/// The Usual over `pieces` (clipped to `usual_range`, in time order), or `None` when it is not
/// comparable: the Period has not started, there is no history, or no Day had Timeline.
#[must_use]
pub fn compute(pieces: &[Piece], scaling: &Scaling) -> Option<UsualTimeline> {
    if !scaling.is_comparable() {
        return None;
    }
    let mut by_day: BTreeMap<NaiveDate, Vec<Piece>> = BTreeMap::new();
    for piece in pieces {
        by_day.entry(piece.date).or_default().push(piece.clone());
    }
    let days: Vec<Figures> = by_day.values().map(|day| figures::measure(day)).collect();
    if days.is_empty() {
        return None;
    }
    let figure = |pick: fn(&Figures) -> i64, additive: bool| {
        let values: Vec<i64> = days.iter().map(pick).collect();
        UsualFigure {
            per_day: spread(values.iter().copied()),
            scaled_total: additive.then(|| {
                let total_s = values.iter().sum();
                scaling
                    .apply(Totals {
                        total_s,
                        ..Totals::default()
                    })
                    .total_s
            }),
        }
    };
    Some(UsualTimeline {
        days: i64::try_from(days.len()).unwrap_or(i64::MAX),
        active: figure(|f| f.seconds.active, true),
        away: figure(|f| f.seconds.away, true),
        unknown: figure(|f| f.seconds.unknown, true),
        switches: figure(|f| f.switches, true),
        longest_focus: figure(|f| f.focus.longest_secs, false),
    })
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::*;
    use crate::report::period::{Period, PeriodKind};
    use crate::timeline::model::Away;
    use crate::timeline_summary::test_support::{active_s, at, piece_s};

    const DAY: i64 = 86_400;

    fn week() -> Period {
        let (from, to) = ("2026-10-12".parse().unwrap(), "2026-10-18".parse().unwrap());
        Period::new(PeriodKind::Week, from, to, chrono_tz::Tz::UTC).unwrap()
    }

    fn history() -> Vec<Piece> {
        // 2026-10-05: 1 h active. 2026-10-06: 2 h active, 10 min away.
        let base = -12 * DAY;
        vec![
            active_s("Code", base, base + 3600),
            active_s("Code", base + DAY, base + DAY + 7200),
            piece_s("Lock", base + DAY + 7200, base + DAY + 7800, Away::Yes),
        ]
    }

    fn scaling(now: DateTime<Utc>, first: Option<DateTime<Utc>>) -> Scaling {
        Scaling::new(&week(), now, first)
    }

    #[test]
    fn averages_are_over_the_days_that_have_timeline() {
        let first = at(0, -12 * DAY);
        let usual = compute(&history(), &scaling(at(0, 30 * DAY), Some(first))).unwrap();
        assert_eq!(usual.days, 2);
        let spread = |f: UsualFigure| (f.per_day.avg_s, f.per_day.min_s, f.per_day.max_s);
        assert_eq!(spread(usual.active), (5400, 3600, 7200));
        assert_eq!(spread(usual.away), (300, 0, 600));
        assert_eq!(spread(usual.longest_focus), (5400, 3600, 7200));
    }

    #[test]
    fn totals_are_scaled_to_the_days_passed_against_history_available() {
        // History starts 10-05: 7 days available; only 3 days of the week have passed.
        let first = at(0, -12 * DAY);
        let now = at(0, -2 * DAY + 3600); // 2026-10-15, the Period's 4th Day.
        let usual = compute(&history(), &scaling(now, Some(first))).unwrap();
        // 10800 * 4 / 7 = 6171.4
        assert_eq!(usual.active.scaled_total, Some(6171));
        assert_eq!(usual.longest_focus.scaled_total, None);
    }

    #[test]
    fn not_comparable_gives_none() {
        let first = Some(at(0, -12 * DAY));
        assert!(
            compute(&history(), &scaling(at(0, -30 * DAY), first)).is_none(),
            "period not started"
        );
        assert!(
            compute(&history(), &scaling(at(0, 30 * DAY), None)).is_none(),
            "no history"
        );
        assert!(
            compute(&[], &scaling(at(0, 30 * DAY), first)).is_none(),
            "no days with timeline"
        );
    }
}
