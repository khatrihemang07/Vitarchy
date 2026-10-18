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

#[test]
fn a_366_day_period_is_accepted_and_a_367_day_one_is_not() {
    let tz = chrono_tz::UTC;
    // 2028 is a leap year: 366 Days.
    let ok = Period::new(PeriodKind::Custom, date(2028, 1, 1), date(2028, 12, 31), tz).unwrap();
    assert_eq!(ok.days().len(), 366);
    let err = Period::new(PeriodKind::Custom, date(2028, 1, 1), date(2029, 1, 1), tz).unwrap_err();
    assert_eq!(err, PeriodError::TooLong);
}

#[test]
fn a_month_must_be_a_whole_calendar_month_including_leap_february() {
    let tz = chrono_tz::UTC;
    let month = |from, to| Period::new(PeriodKind::Month, from, to, tz);
    assert!(month(date(2028, 2, 1), date(2028, 2, 29)).is_ok());
    assert_eq!(
        month(date(2028, 2, 1), date(2028, 2, 28)).unwrap_err(),
        PeriodError::KindMismatch
    );
    assert!(month(date(2027, 2, 1), date(2027, 2, 28)).is_ok());
    assert_eq!(
        month(date(2027, 2, 1), date(2027, 3, 1)).unwrap_err(),
        PeriodError::KindMismatch
    );
}

#[test]
fn the_previous_period_of_a_leap_march_is_a_29_day_february() {
    let march = period(PeriodKind::Month, date(2028, 3, 1), date(2028, 3, 31)).previous();
    assert_eq!(
        (march.from, march.to),
        (date(2028, 2, 1), date(2028, 2, 29))
    );
    let march_2027 = period(PeriodKind::Month, date(2027, 3, 1), date(2027, 3, 31)).previous();
    assert_eq!(
        (march_2027.from, march_2027.to),
        (date(2027, 2, 1), date(2027, 2, 28))
    );
}

#[test]
fn the_previous_period_crosses_the_year_for_every_kind() {
    let span = |p: Period| (p.from, p.to);
    let day = period(PeriodKind::Day, date(2026, 1, 1), date(2026, 1, 1)).previous();
    assert_eq!(span(day), (date(2025, 12, 31), date(2025, 12, 31)));
    let week = period(PeriodKind::Week, date(2026, 1, 5), date(2026, 1, 11)).previous();
    assert_eq!(span(week), (date(2025, 12, 29), date(2026, 1, 4)));
    let month = period(PeriodKind::Month, date(2026, 1, 1), date(2026, 1, 31)).previous();
    assert_eq!(span(month), (date(2025, 12, 1), date(2025, 12, 31)));
    let custom = period(PeriodKind::Custom, date(2026, 1, 2), date(2026, 1, 4)).previous();
    assert_eq!(span(custom), (date(2025, 12, 30), date(2026, 1, 1)));
}

#[test]
fn the_previous_period_keeps_the_kind_and_time_zone() {
    let tz: Tz = "Asia/Kolkata".parse().unwrap();
    let p = Period::new(PeriodKind::Week, date(2026, 10, 12), date(2026, 10, 18), tz).unwrap();
    let before = p.previous();
    assert_eq!((before.kind, before.tz), (PeriodKind::Week, tz));
    // Its range ends where this Period's begins, so no instant is in both.
    assert_eq!(before.utc_range().1, p.utc_range().0);
}

#[test]
fn a_custom_previous_period_of_a_365_day_stretch_is_the_365_days_before() {
    let p = period(PeriodKind::Custom, date(2026, 1, 1), date(2026, 12, 31)).previous();
    assert_eq!((p.from, p.to), (date(2025, 1, 1), date(2025, 12, 31)));
}

#[test]
fn the_usual_range_is_the_28_days_before_the_period() {
    let p = period(PeriodKind::Week, date(2026, 10, 12), date(2026, 10, 18));
    let usual = p.usual_range();
    assert_eq!(
        (usual.from, usual.to),
        (date(2026, 9, 14), date(2026, 10, 11))
    );
    assert_eq!(usual.days().len(), 28);
}

#[test]
fn the_usual_range_crosses_a_month_boundary() {
    let p = period(PeriodKind::Day, date(2026, 3, 2), date(2026, 3, 2));
    let usual = p.usual_range();
    assert_eq!((usual.from, usual.to), (date(2026, 2, 2), date(2026, 3, 1)));
}

#[test]
fn the_usual_range_has_27_local_days_of_24_hours_and_one_of_23_across_spring_dst() {
    let tz: Tz = "Europe/Berlin".parse().unwrap();
    // Berlin skips an hour on 2026-03-29, which falls inside 2026-03-08..2026-04-04.
    let p = Period::new(PeriodKind::Day, date(2026, 4, 5), date(2026, 4, 5), tz).unwrap();
    let usual = p.usual_range();
    assert_eq!((usual.from, usual.to), (date(2026, 3, 8), date(2026, 4, 4)));
    let (from, to) = usual.utc_range();
    assert_eq!((to - from).num_hours(), 28 * 24 - 1);
    assert_eq!(from.to_rfc3339(), "2026-03-07T23:00:00+00:00");
    assert_eq!(to.to_rfc3339(), "2026-04-04T22:00:00+00:00");
}

#[test]
fn previous_containing_a_day_keeps_the_day_alignment() {
    let day = period(PeriodKind::Day, date(2026, 10, 12), date(2026, 10, 12));
    let reached = day.previous_containing(date(2026, 10, 9));
    assert_eq!(
        (reached.from, reached.to),
        (date(2026, 10, 9), date(2026, 10, 9))
    );
}

#[test]
fn previous_containing_a_day_steps_whole_weeks() {
    let week = period(PeriodKind::Week, date(2026, 10, 26), date(2026, 11, 1));
    let reached = week.previous_containing(date(2026, 10, 9));
    assert_eq!(
        (reached.from, reached.to),
        (date(2026, 10, 5), date(2026, 10, 11))
    );
}

#[test]
fn previous_containing_a_day_steps_whole_months_across_a_year_boundary() {
    let month = period(PeriodKind::Month, date(2026, 2, 1), date(2026, 2, 28));
    let reached = month.previous_containing(date(2025, 11, 20));
    assert_eq!(
        (reached.from, reached.to),
        (date(2025, 11, 1), date(2025, 11, 30))
    );
}

#[test]
fn previous_containing_a_day_steps_custom_stretches_by_their_length() {
    let custom = period(PeriodKind::Custom, date(2026, 10, 10), date(2026, 10, 14));
    let reached = custom.previous_containing(date(2026, 9, 28));
    assert_eq!(
        (reached.from, reached.to),
        (date(2026, 9, 25), date(2026, 9, 29))
    );
    assert_eq!(reached.kind, PeriodKind::Custom);
}

#[test]
fn previous_containing_the_immediate_previous_day_is_previous() {
    let week = period(PeriodKind::Week, date(2026, 10, 12), date(2026, 10, 18));
    assert_eq!(week.previous_containing(date(2026, 10, 7)), week.previous());
}
