//! The Usual: how a group usually went over the Usual's range before a Period.
//!
//! The Usual's groups come from the same tree builder as the Period's; this module scales
//! them to the Days that have passed (see `scaling`) and compares like for like.

use serde::Serialize;
use utoipa::ToSchema;

use super::{
    compare::{StatsChange, change, walk_matches},
    model::{Group, Overview, OverviewChange},
    overview,
    scaling::{Scaling, Totals},
    stats::{Spread, Stats},
};

/// What a group usually looks like per Day and per entry, over the Days it happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct UsualFigures {
    /// Days in the Usual's range on which the group had time.
    pub days: i64,
    /// Spread of its daily totals over those Days (not scaled).
    pub per_day: Spread,
    /// Spread of each entry's time over the Usual's range (not scaled).
    pub per_entry: Spread,
}

impl From<&Stats> for UsualFigures {
    fn from(stats: &Stats) -> Self {
        Self {
            days: stats.days,
            per_day: stats.per_day,
            per_entry: stats.per_entry,
        }
    }
}

/// How a group compares with the Usual.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, ToSchema)]
pub struct UsualComparison {
    /// Current minus the Usual. Totals, Days and entries are against the Usual scaled to
    /// the Days that have passed; the spreads are against the Usual's own. Absent when `is_new`.
    pub change: Option<StatsChange>,
    /// True when the group had no time in the Usual's range.
    pub is_new: bool,
    /// The Usual itself, so the Day table can show it; absent when `is_new`.
    pub usual: Option<UsualFigures>,
}

/// The Usual's total, Days and entries for the Days that have passed. Spreads stay as they
/// are: an average per Day does not grow with the Period.
#[must_use]
pub fn scaled(usual: &Stats, scaling: &Scaling) -> Stats {
    let totals = scaling.apply(Totals::from(usual));
    Stats {
        days: totals.days,
        entries: totals.entries,
        total_s: totals.total_s,
        ..*usual
    }
}

/// Fills `vs_usual` on each group (and its children) from the Usual's groups. When the
/// Period has not started there is nothing to compare yet, so the totals show no change.
pub fn attach_usual(groups: &mut [Group], usual: &[Group], scaling: &Scaling) {
    walk_matches(groups, usual, &mut |group, matched| {
        group.vs_usual = match matched {
            Some(m) => UsualComparison {
                change: Some(if scaling.elapsed_days == 0 {
                    StatsChange::default()
                } else {
                    change(&group.stats, &scaled(&m.stats, scaling))
                }),
                is_new: false,
                usual: Some(UsualFigures::from(&m.stats)),
            },
            None => UsualComparison {
                is_new: true,
                ..UsualComparison::default()
            },
        };
    });
}

/// The overview's change from the Usual scaled to the Days that have passed (the per-Day
/// average is not scaled). No change when there is nothing to compare with.
#[must_use]
pub fn overview_change(now: &Overview, usual: &Overview, scaling: &Scaling) -> OverviewChange {
    if !scaling.is_comparable() {
        return OverviewChange::default();
    }
    let totals = scaling.apply(Totals::from(usual));
    let scaled = Overview {
        total_s: totals.total_s,
        entries: totals.entries,
        days_tracked: totals.days,
        ..*usual
    };
    overview::change(now, &scaled)
}
