//! Resolving the Previous Period of a Report: the closest earlier Period of the same kind
//! with entries matching the filters, falling back to the calendar neighbour.

use sqlx::PgPool;

use crate::{
    repo::entries,
    report::{
        entry::WindowEntry,
        period::Period,
        previous::{latest_tracked_day, lookback_range},
        query::ReportQuery,
    },
};

async fn read(pool: &PgPool, period: &Period) -> Result<Vec<WindowEntry>, sqlx::Error> {
    let (from, to) = period.utc_range();
    entries::in_range(pool, from, to).await
}

/// Reads the calendar neighbour; when nothing in it has time matching the filters, looks back
/// once over [`lookback_range`] for the last tracked Day and reads the Period that holds it.
/// With nothing in the lookback either, the neighbour stays (everything reads as new).
/// Returns the Period compared with and the entries overlapping it.
pub async fn resolve(
    pool: &PgPool,
    query: &ReportQuery,
) -> Result<(Period, Vec<WindowEntry>), sqlx::Error> {
    let immediate = query.period.previous();
    let entries = read(pool, &immediate).await?;
    if latest_tracked_day(&entries, &query.filter, &immediate).is_some() {
        return Ok((immediate, entries));
    }
    let lookback = lookback_range(&immediate);
    let candidates = read(pool, &lookback).await?;
    tracing::debug!(
        immediate_entries = entries.len(),
        lookback_entries = candidates.len(),
        "previous period looked back"
    );
    let Some(day) = latest_tracked_day(&candidates, &query.filter, &lookback) else {
        return Ok((immediate, entries));
    };
    let period = query.period.previous_containing(day);
    let entries = read(pool, &period).await?;
    Ok((period, entries))
}
