//! When the computer is in use: seconds by weekday and hour of the local day.

use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use chrono_tz::Tz;
use serde::Serialize;
use utoipa::ToSchema;

use super::{ActivityRow, seconds};
use crate::analysis::{Period, split_by_day};

/// Rows of the grid: Monday first.
pub const WEEKDAYS: usize = 7;
/// Columns of the grid.
pub const HOURS: usize = 24;

/// Seconds of Activity by weekday and hour.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct Heatmap {
    /// `seconds[weekday][hour]`, weekday 0 = Monday, hour 0..=23 local time.
    pub seconds: Vec<Vec<i64>>,
}

/// Adds `[start, end)` to `grid`, cutting it at every local hour boundary.
fn add_span(grid: &mut [[i64; HOURS]; WEEKDAYS], tz: Tz, start: DateTime<Utc>, end: DateTime<Utc>) {
    let mut cursor = start;
    while cursor < end {
        let local = cursor.with_timezone(&tz).naive_local();
        let into_hour = Duration::minutes(i64::from(local.minute()))
            + Duration::seconds(i64::from(local.second()))
            + Duration::nanoseconds(i64::from(local.nanosecond()));
        // Always positive: `into_hour` is under an hour.
        let next = (cursor + (Duration::hours(1) - into_hour)).min(end);
        let weekday = local.weekday().num_days_from_monday() as usize;
        grid[weekday][local.hour() as usize] += (next - cursor).num_milliseconds();
        cursor = next;
    }
}

/// The grid for the rows' portions inside the Period.
#[must_use]
pub fn heatmap(period: &Period, rows: &[ActivityRow]) -> Heatmap {
    let mut grid = [[0_i64; HOURS]; WEEKDAYS];
    for row in rows {
        for portion in split_by_day(period, row.start, row.end) {
            add_span(&mut grid, period.tz(), portion.start, portion.end);
        }
    }
    Heatmap {
        seconds: grid
            .iter()
            .map(|day| day.iter().map(|ms| seconds(*ms)).collect())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::row::at;
    use super::*;

    fn grid(tz: &str, from: &str, to: &str, spans: &[(&str, &str)]) -> Vec<Vec<i64>> {
        let period = Period::parse(from, to, tz).unwrap();
        let rows: Vec<ActivityRow> = spans
            .iter()
            .map(|(s, e)| ActivityRow::new("App", "", at(s), at(e)))
            .collect();
        heatmap(&period, &rows).seconds
    }

    #[test]
    fn the_grid_is_seven_weekdays_by_twenty_four_hours() {
        let g = grid("UTC", "2026-05-01", "2026-05-01", &[]);
        assert_eq!((g.len(), g[0].len()), (7, 24));
        assert!(g.iter().flatten().all(|s| *s == 0));
    }

    #[test]
    fn a_span_lands_in_its_weekday_and_hour() {
        // 2026-05-01 is a Friday (index 4).
        let g = grid(
            "UTC",
            "2026-05-01",
            "2026-05-01",
            &[("2026-05-01T09:10:00Z", "2026-05-01T09:40:00Z")],
        );
        assert_eq!(g[4][9], 30 * 60);
        assert_eq!(g.iter().flatten().sum::<i64>(), 30 * 60);
    }

    #[test]
    fn a_span_over_an_hour_boundary_is_cut_there() {
        let g = grid(
            "UTC",
            "2026-05-01",
            "2026-05-01",
            &[("2026-05-01T09:50:00Z", "2026-05-01T10:20:00Z")],
        );
        assert_eq!((g[4][9], g[4][10]), (10 * 60, 20 * 60));
    }

    #[test]
    fn a_span_over_midnight_spills_into_the_next_weekday() {
        let g = grid(
            "UTC",
            "2026-05-01",
            "2026-05-02",
            &[("2026-05-01T23:30:00Z", "2026-05-02T00:30:00Z")],
        );
        assert_eq!((g[4][23], g[5][0]), (30 * 60, 30 * 60));
    }

    #[test]
    fn hours_are_local_hours_not_utc_hours() {
        // Kolkata is UTC+5:30: 19:15Z is 00:45 there, so the cut falls at 19:30Z.
        let g = grid(
            "Asia/Kolkata",
            "2026-05-02",
            "2026-05-02",
            &[("2026-05-01T19:15:00Z", "2026-05-01T19:45:00Z")],
        );
        assert_eq!((g[5][0], g[5][1]), (15 * 60, 15 * 60));
    }

    #[test]
    fn a_multi_day_span_fills_every_hour_it_covers() {
        let g = grid(
            "UTC",
            "2026-05-01",
            "2026-05-07",
            &[("2026-05-01T00:00:00Z", "2026-05-03T00:00:00Z")],
        );
        assert!(g[4].iter().all(|s| *s == 3600));
        assert!(g[5].iter().all(|s| *s == 3600));
        assert!(g[6].iter().all(|s| *s == 0));
    }

    #[test]
    fn the_part_outside_the_period_is_left_out() {
        let g = grid(
            "UTC",
            "2026-05-02",
            "2026-05-02",
            &[("2026-05-01T23:00:00Z", "2026-05-02T01:00:00Z")],
        );
        assert_eq!(g.iter().flatten().sum::<i64>(), 3600);
        assert_eq!(g[5][0], 3600);
    }

    #[test]
    fn the_whole_grid_sums_to_the_time_in_the_period() {
        let g = grid(
            "America/New_York",
            "2026-03-07",
            "2026-03-09",
            &[("2026-03-07T10:13:07Z", "2026-03-09T01:41:59Z")],
        );
        assert_eq!(g.iter().flatten().sum::<i64>(), 39 * 3600 + 28 * 60 + 52);
    }
}
