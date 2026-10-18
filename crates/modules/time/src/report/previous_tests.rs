//! `latest_tracked_day` over in-memory entries.

use chrono::NaiveDate;

use super::{
    entry::fixtures::entry,
    filter::Filter,
    period::{Period, PeriodKind},
    previous::{latest_tracked_day, lookback_range},
};

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn range(from: NaiveDate, to: NaiveDate) -> Period {
    Period::new(PeriodKind::Custom, from, to, chrono_tz::UTC).unwrap()
}

fn filter(text: &str) -> Filter {
    Filter {
        search: (!text.is_empty()).then(|| text.to_owned()),
        ..Filter::default()
    }
}

#[test]
fn the_latest_matching_day_in_the_range_wins() {
    let entries = [
        entry(1, "2026-10-05T08:00:00Z", 1, "work"),
        entry(2, "2026-10-09T08:00:00Z", 1, "work"),
        entry(3, "2026-10-07T08:00:00Z", 1, "work"),
    ];
    let found = latest_tracked_day(
        &entries,
        &filter(""),
        &range(date(2026, 10, 1), date(2026, 10, 11)),
    );
    assert_eq!(found, Some(date(2026, 10, 9)));
}

#[test]
fn only_entries_matching_the_filter_count() {
    let entries = [
        entry(1, "2026-10-05T08:00:00Z", 1, "gym"),
        entry(2, "2026-10-09T08:00:00Z", 1, "work"),
    ];
    let found = latest_tracked_day(
        &entries,
        &filter("gym"),
        &range(date(2026, 10, 1), date(2026, 10, 11)),
    );
    assert_eq!(found, Some(date(2026, 10, 5)));
}

#[test]
fn an_entry_crossing_midnight_counts_for_the_day_it_reaches() {
    let entries = [entry(1, "2026-10-09T22:00:00Z", 4, "night")];
    let found = latest_tracked_day(
        &entries,
        &filter(""),
        &range(date(2026, 10, 1), date(2026, 10, 11)),
    );
    assert_eq!(found, Some(date(2026, 10, 10)));
}

#[test]
fn time_outside_the_range_does_not_count() {
    let entries = [entry(1, "2026-10-09T22:00:00Z", 4, "night")];
    let found = latest_tracked_day(
        &entries,
        &filter(""),
        &range(date(2026, 10, 1), date(2026, 10, 9)),
    );
    assert_eq!(found, Some(date(2026, 10, 9)));
}

#[test]
fn nothing_matching_gives_none() {
    let entries = [entry(1, "2026-10-09T08:00:00Z", 1, "work")];
    let found = latest_tracked_day(
        &entries,
        &filter("gym"),
        &range(date(2026, 10, 1), date(2026, 10, 11)),
    );
    assert_eq!(found, None);
    assert_eq!(
        latest_tracked_day(
            &[],
            &filter(""),
            &range(date(2026, 10, 1), date(2026, 10, 11))
        ),
        None
    );
}

#[test]
fn the_lookback_is_a_full_period_length_of_days_before_the_immediate_period() {
    let immediate = range(date(2026, 10, 5), date(2026, 10, 11));
    let lookback = lookback_range(&immediate);
    assert_eq!(lookback.to, date(2026, 10, 4));
    assert_eq!(lookback.days().len(), 366);
}
