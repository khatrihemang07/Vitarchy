//! Week-over-week screen time for the busiest apps.

use std::collections::HashMap;

use chrono::{Datelike, Days, NaiveDate};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::{ActivityRow, seconds, totals::slot};
use crate::analysis::{Period, split_by_day};

/// Apps drawn on their own; the rest fold into `other`.
pub const TOP_APPS: usize = 5;

/// Which weekday a week starts on (the Core week-start Setting).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum WeekStart {
    /// Weeks run Monday to Sunday.
    #[default]
    Monday,
    /// Weeks run Sunday to Saturday.
    Sunday,
}

impl WeekStart {
    /// The first day of the week that holds `date`.
    #[must_use]
    pub fn week_of(self, date: NaiveDate) -> NaiveDate {
        let back = match self {
            Self::Monday => date.weekday().num_days_from_monday(),
            Self::Sunday => date.weekday().num_days_from_sunday(),
        };
        date.checked_sub_days(Days::new(u64::from(back)))
            .unwrap_or(date)
    }
}

/// One week of the trend.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct TrendWeek {
    /// Local date the week starts on.
    #[schema(value_type = String, format = Date)]
    pub start: NaiveDate,
    /// How many of its Days lie inside the Period (under 7 at the ends).
    pub days: u32,
}

/// One app's seconds in each week.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct TrendSeries {
    /// The app.
    pub app: String,
    /// Seconds per week, in the order of `weeks`.
    pub seconds: Vec<i64>,
}

/// Weekly screen time over a Period.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct Trend {
    /// Weeks from the one holding the Period's first Day to the one holding its last.
    pub weeks: Vec<TrendWeek>,
    /// The longest-used apps, longest first.
    pub series: Vec<TrendSeries>,
    /// Seconds per week of every other app.
    pub other: Vec<i64>,
}

/// Weekly totals for the [`TOP_APPS`] apps and the rest.
#[must_use]
pub fn trend(period: &Period, rows: &[ActivityRow], week_start: WeekStart) -> Trend {
    let first = period
        .days()
        .first()
        .map_or(NaiveDate::MIN, |d| week_start.week_of(d.date));
    let mut weeks: Vec<TrendWeek> = Vec::new();
    for day in period.days() {
        let start = week_start.week_of(day.date);
        match weeks.last_mut() {
            Some(week) if week.start == start => week.days += 1,
            _ => weeks.push(TrendWeek { start, days: 1 }),
        }
    }
    let mut per_app: HashMap<String, Vec<i64>> = HashMap::new();
    for row in rows {
        for portion in split_by_day(period, row.start, row.end) {
            let index = usize::try_from((week_start.week_of(portion.date) - first).num_days() / 7)
                .unwrap_or(0);
            let cells = slot(&mut per_app, &row.app);
            cells.resize(weeks.len(), 0);
            cells[index] += portion.millis();
        }
    }
    let mut ranked: Vec<(String, Vec<i64>)> = per_app.into_iter().collect();
    ranked.sort_by(|a, b| {
        let (ta, tb) = (a.1.iter().sum::<i64>(), b.1.iter().sum::<i64>());
        tb.cmp(&ta).then_with(|| a.0.cmp(&b.0))
    });
    let rest = ranked.split_off(ranked.len().min(TOP_APPS));
    let mut other = vec![0_i64; weeks.len()];
    for (_, cells) in &rest {
        for (total, cell) in other.iter_mut().zip(cells) {
            *total += cell;
        }
    }
    Trend {
        weeks,
        series: ranked
            .into_iter()
            .map(|(app, cells)| TrendSeries {
                app,
                seconds: cells.into_iter().map(seconds).collect(),
            })
            .collect(),
        other: other.into_iter().map(seconds).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::row::at;
    use super::*;

    fn d(text: &str) -> NaiveDate {
        text.parse().unwrap()
    }

    fn row(app: &str, start: &str, end: &str) -> ActivityRow {
        ActivityRow::new(app, "", at(start), at(end))
    }

    #[test]
    fn a_week_starts_on_the_chosen_weekday() {
        // 2026-05-06 is a Wednesday.
        assert_eq!(WeekStart::Monday.week_of(d("2026-05-06")), d("2026-05-04"));
        assert_eq!(WeekStart::Sunday.week_of(d("2026-05-06")), d("2026-05-03"));
        assert_eq!(WeekStart::Monday.week_of(d("2026-05-03")), d("2026-04-27"));
        assert_eq!(WeekStart::Sunday.week_of(d("2026-05-03")), d("2026-05-03"));
    }

    #[test]
    fn weeks_cover_the_period_and_count_the_days_inside_it() {
        let period = Period::parse("2026-05-06", "2026-05-12", "UTC").unwrap();
        let result = trend(&period, &[], WeekStart::Monday);
        let weeks: Vec<(String, u32)> = result
            .weeks
            .iter()
            .map(|w| (w.start.to_string(), w.days))
            .collect();
        assert_eq!(
            weeks,
            [("2026-05-04".to_owned(), 5), ("2026-05-11".to_owned(), 2)]
        );
        assert!(result.series.is_empty());
        assert_eq!(result.other, [0, 0]);
    }

    #[test]
    fn sunday_weeks_split_the_same_period_differently() {
        let period = Period::parse("2026-05-06", "2026-05-12", "UTC").unwrap();
        let weeks: Vec<u32> = trend(&period, &[], WeekStart::Sunday)
            .weeks
            .iter()
            .map(|w| w.days)
            .collect();
        assert_eq!(weeks, [4, 3]);
    }

    #[test]
    fn each_app_gets_its_seconds_per_week_longest_first() {
        let period = Period::parse("2026-05-04", "2026-05-17", "UTC").unwrap();
        let rows = [
            row("Code", "2026-05-04T09:00:00Z", "2026-05-04T10:00:00Z"),
            row("Code", "2026-05-12T09:00:00Z", "2026-05-12T11:00:00Z"),
            row("Spotify", "2026-05-05T09:00:00Z", "2026-05-05T09:30:00Z"),
        ];
        let result = trend(&period, &rows, WeekStart::Monday);
        let series: Vec<(&str, &[i64])> = result
            .series
            .iter()
            .map(|s| (s.app.as_str(), s.seconds.as_slice()))
            .collect();
        assert_eq!(
            series,
            [("Code", &[3600, 7200][..]), ("Spotify", &[1800, 0][..])]
        );
        assert_eq!(result.other, [0, 0]);
    }

    #[test]
    fn apps_beyond_the_top_five_fold_into_other() {
        let period = Period::parse("2026-05-04", "2026-05-10", "UTC").unwrap();
        let rows: Vec<ActivityRow> = (1..=7)
            .map(|i| {
                row(
                    &format!("App{i}"),
                    "2026-05-04T08:00:00Z",
                    &format!("2026-05-04T{:02}:00:00Z", 8 + i),
                )
            })
            .collect();
        let result = trend(&period, &rows, WeekStart::Monday);
        let names: Vec<&str> = result.series.iter().map(|s| s.app.as_str()).collect();
        assert_eq!(names, ["App7", "App6", "App5", "App4", "App3"]);
        assert_eq!(result.other, [3 * 3600]);
    }

    #[test]
    fn a_row_over_a_week_boundary_is_shared_between_the_weeks() {
        let period = Period::parse("2026-05-04", "2026-05-17", "UTC").unwrap();
        let rows = [row("Code", "2026-05-10T23:00:00Z", "2026-05-11T02:00:00Z")];
        let result = trend(&period, &rows, WeekStart::Monday);
        assert_eq!(result.series[0].seconds, [3600, 7200]);
    }
}
