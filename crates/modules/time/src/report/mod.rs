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
pub mod previous;
pub mod query;
mod scaling;
mod series;
pub mod stats;
mod tree;
pub mod usual;

#[cfg(test)]
mod build_tests;
#[cfg(test)]
mod build_usual_tests;
#[cfg(test)]
mod compare_tests;
#[cfg(test)]
mod period_tests;
#[cfg(test)]
mod previous_tests;
#[cfg(test)]
mod query_tests;
#[cfg(test)]
mod scaling_tests;
#[cfg(test)]
mod usual_tests;

use chrono::{DateTime, Utc};
use entry::WindowEntry;
use grouping::Grouping;
use model::{Group, Overview, PeriodInfo, Report};
use period::Period;
use query::ReportQuery;
use scaling::Scaling;

/// The entries a Report reads, each set as read for its own range of Days.
#[derive(Debug, Clone, Copy)]
pub struct Entries<'a> {
    /// Entries overlapping the Period.
    pub current: &'a [WindowEntry],
    /// The Previous Period the Report compares with, and its entries.
    pub previous: Reference<'a>,
    /// Entries overlapping the Usual's range.
    pub usual: &'a [WindowEntry],
}

/// A Period a Report compares with, and the entries read for it. The route resolves it,
/// because finding the last Period with entries needs reads the pure code cannot make.
#[derive(Debug, Clone, Copy)]
pub struct Reference<'a> {
    /// The Period compared with.
    pub period: Period,
    /// Entries overlapping it.
    pub entries: &'a [WindowEntry],
}

/// What a Report needs to know besides the entries, injected so tests are deterministic.
#[derive(Debug, Clone, Copy)]
pub struct Context {
    /// The moment of the request; "today" is its Day in the Period's time zone.
    pub now: DateTime<Utc>,
    /// Start of the person's earliest Time Entry, whatever the filters; `None` with none.
    pub first_entry: Option<DateTime<Utc>>,
}

/// Groups and overview of a reference range (the Previous Period or the Usual's).
fn summarize(
    entries: &[WindowEntry],
    query: &ReportQuery,
    period: &Period,
    groupings: &[Grouping],
) -> (Vec<Group>, Overview) {
    let items = items::prepare(entries, &query.filter, period);
    (
        tree::build(&refs(&items), groupings, &period.days()),
        overview::overview(&items),
    )
}

/// Builds the Report for `query`. Filtering and clipping to each range happen here.
#[must_use]
pub fn build_report(query: &ReportQuery, entries: &Entries<'_>, context: &Context) -> Report {
    let earlier = entries.previous.period;
    let usual_period = query.period.usual_range();
    let scaling = Scaling::new(&query.period, context.now, context.first_entry);
    let groupings = query.groupings();
    let now = items::prepare(entries.current, &query.filter, &query.period);
    let days = query.period.days();

    let mut groups = tree::build(&refs(&now), &groupings, &days);
    let (before_groups, before_overview) =
        summarize(entries.previous.entries, query, &earlier, &groupings);
    let (usual_groups, usual_overview) = summarize(entries.usual, query, &usual_period, &groupings);
    compare::attach_previous(&mut groups, &before_groups);
    usual::attach_usual(&mut groups, &usual_groups, &scaling);

    let mut overview = overview::overview(&now);
    overview.vs_previous = overview::change(&overview, &before_overview);
    overview.vs_usual = usual::overview_change(&overview, &usual_overview, &scaling);
    overview.usual_days = usual_overview.days_tracked;

    Report {
        period: PeriodInfo {
            kind: query.period.kind,
            from: query.period.from,
            to: query.period.to,
            previous_from: earlier.from,
            previous_to: earlier.to,
            previous_skipped: earlier != query.period.previous(),
            usual_from: usual_period.from,
            usual_to: usual_period.to,
            elapsed_days: scaling.elapsed_days,
            usual_available_days: scaling.available_days,
        },
        overview,
        daily: series::daily_series(&days, now.iter().flat_map(|i| i.parts.iter())),
        group1: query.group1,
        group2: query.group2,
        groups,
    }
}

fn refs<'a, 'b>(items: &'b [items::Item<'a>]) -> Vec<&'b items::Item<'a>> {
    items.iter().collect()
}
