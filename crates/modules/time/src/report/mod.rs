//! Report: the pure computation behind `GET /api/time/report`. Entries come in already
//! read from the database; nothing here touches storage.

pub mod compare;
pub mod day_split;
pub mod entry;
pub mod facets;
pub mod filter;
pub mod grouping;
mod items;
pub mod model;
mod overview;
pub mod period;
pub mod query;
mod series;
pub mod stats;
mod tree;

#[cfg(test)]
mod build_tests;
#[cfg(test)]
mod period_tests;
#[cfg(test)]
mod query_tests;

use entry::WindowEntry;
use model::{PeriodInfo, Report};
use query::ReportQuery;

/// Builds the Report for `query`. `current` holds the entries overlapping the Period and
/// `previous` those overlapping the Previous Period; filtering and clipping happen here.
#[must_use]
pub fn build_report(
    query: &ReportQuery,
    current: &[WindowEntry],
    previous: &[WindowEntry],
) -> Report {
    let earlier = query.period.previous();
    let now = items::prepare(current, &query.filter, &query.period);
    let before = items::prepare(previous, &query.filter, &earlier);
    let days = query.period.days();
    let groupings = query.groupings();

    let now_refs: Vec<&items::Item<'_>> = now.iter().collect();
    let before_refs: Vec<&items::Item<'_>> = before.iter().collect();
    let earlier_days = earlier.days();

    let mut groups = tree::build(&now_refs, &groupings, &days);
    compare::attach_changes(
        &mut groups,
        &tree::build(&before_refs, &groupings, &earlier_days),
    );

    let before_overview = overview::overview(&before, None);
    Report {
        period: PeriodInfo {
            kind: query.period.kind,
            from: query.period.from,
            to: query.period.to,
            previous_from: earlier.from,
            previous_to: earlier.to,
        },
        overview: overview::overview(&now, Some(&before_overview)),
        daily: series::daily_series(&days, now.iter().flat_map(|i| i.parts.iter())),
        group1: query.group1,
        group2: query.group2,
        groups,
    }
}
