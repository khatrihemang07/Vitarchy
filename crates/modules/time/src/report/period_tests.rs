//! Tests for `period`.

use chrono::NaiveDate;
use chrono_tz::Tz;

use super::period::*;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}
fn period(kind: PeriodKind, from: NaiveDate, to: NaiveDate) -> Period {
    Period::new(kind, from, to, chrono_tz::UTC).unwrap()
}

#[test]
fn the_range_runs_from_local_midnight_to_the_next_local_midnight() {
    let tz: Tz = "Europe/Berlin".parse().unwrap();
    let p = Period::new(PeriodKind::Day, date(2026, 10, 17), date(2026, 10, 17), tz).unwrap();
    let (from, to) = p.utc_range();
    assert_eq!(from.to_rfc3339(), "2026-10-16T22:00:00+00:00");
    assert_eq!(to.to_rfc3339(), "2026-10-17T22:00:00+00:00");
}

#[test]
fn a_dst_change_makes_the_range_23_or_25_hours() {
    let tz: Tz = "Europe/Berlin".parse().unwrap();
    let spring = Period::new(PeriodKind::Day, date(2026, 3, 29), date(2026, 3, 29), tz).unwrap();
    let autumn = Period::new(PeriodKind::Day, date(2026, 10, 25), date(2026, 10, 25), tz).unwrap();
    let hours = |p: Period| (p.utc_range().1 - p.utc_range().0).num_hours();
    assert_eq!((hours(spring), hours(autumn)), (23, 25));
}

#[test]
fn a_midnight_skipped_by_dst_starts_the_day_at_the_first_hour_that_exists() {
    let tz: Tz = "America/Sao_Paulo".parse().unwrap();
    // Brazil skipped 2018-11-04 00:00 -> 01:00 local.
    let start = local_midnight(tz, date(2018, 11, 4));
    assert_eq!(start.to_rfc3339(), "2018-11-04T03:00:00+00:00");
}

#[test]
fn previous_steps_back_by_the_kind() {
    let day = period(PeriodKind::Day, date(2026, 3, 1), date(2026, 3, 1)).previous();
    assert_eq!((day.from, day.to), (date(2026, 2, 28), date(2026, 2, 28)));
    let week = period(PeriodKind::Week, date(2026, 10, 12), date(2026, 10, 18)).previous();
    assert_eq!(
        (week.from, week.to),
        (date(2026, 10, 5), date(2026, 10, 11))
    );
    let month = period(PeriodKind::Month, date(2026, 3, 1), date(2026, 3, 31)).previous();
    assert_eq!(
        (month.from, month.to),
        (date(2026, 2, 1), date(2026, 2, 28))
    );
    let january = period(PeriodKind::Month, date(2026, 1, 1), date(2026, 1, 31)).previous();
    assert_eq!(
        (january.from, january.to),
        (date(2025, 12, 1), date(2025, 12, 31))
    );
    let custom = period(PeriodKind::Custom, date(2026, 10, 10), date(2026, 10, 14)).previous();
    assert_eq!(
        (custom.from, custom.to),
        (date(2026, 10, 5), date(2026, 10, 9))
    );
}

#[test]
fn days_lists_every_day_inclusive() {
    let days = period(PeriodKind::Custom, date(2026, 2, 27), date(2026, 3, 2)).days();
    assert_eq!(days.len(), 4);
    assert_eq!(days[3], date(2026, 3, 2));
}

#[test]
fn bad_dates_are_refused() {
    let tz = chrono_tz::UTC;
    let new = |kind, from, to| Period::new(kind, from, to, tz).unwrap_err();
    assert_eq!(
        new(PeriodKind::Custom, date(2026, 2, 2), date(2026, 2, 1)),
        PeriodError::Reversed
    );
    assert_eq!(
        new(PeriodKind::Custom, date(2025, 1, 1), date(2026, 1, 2)),
        PeriodError::TooLong
    );
    assert_eq!(
        new(PeriodKind::Day, date(2026, 2, 1), date(2026, 2, 2)),
        PeriodError::KindMismatch
    );
    assert_eq!(
        new(PeriodKind::Week, date(2026, 2, 1), date(2026, 2, 3)),
        PeriodError::KindMismatch
    );
    assert_eq!(
        new(PeriodKind::Month, date(2026, 2, 2), date(2026, 3, 1)),
        PeriodError::KindMismatch
    );
    assert!(Period::new(PeriodKind::Custom, date(2026, 1, 1), date(2026, 12, 31), tz).is_ok());
}
