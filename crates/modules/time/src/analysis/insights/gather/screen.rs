//! Series for the busiest apps and Contexts: screen time per Day.

use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{DateTime, NaiveDate, Utc};

use crate::analysis::{
    Period,
    activity::{ActivityRow, totals},
    insights::{Measure, Point, Series, Subject, SubjectKind},
    split_by_day,
};

/// Milliseconds per Day, per app or Context name.
type PerDay = HashMap<String, BTreeMap<NaiveDate, i64>>;

/// Apps, and Contexts, measured: the busiest of each over the report.
pub(super) const TOP_SUBJECTS: usize = 5;

/// Screen-time series over the reach for the busiest apps and Contexts of `report`. Only Days
/// that finished and have some Activity count, so a Day with no import is not a run of zeros.
pub(super) fn series(
    report: &Period,
    reach: &Period,
    rows: &[ActivityRow],
    now: DateTime<Utc>,
) -> Vec<Series> {
    let (apps, contexts) = busiest(report, rows);
    let mut per_app = PerDay::new();
    let mut per_context = PerDay::new();
    let mut day_total: BTreeMap<NaiveDate, i64> = BTreeMap::new();
    for row in rows {
        for portion in split_by_day(reach, row.start, row.end) {
            let ms = portion.millis();
            *day_total.entry(portion.date).or_default() += ms;
            if apps.contains(&row.app) {
                *per_app
                    .entry(row.app.clone())
                    .or_default()
                    .entry(portion.date)
                    .or_default() += ms;
            }
            if contexts.contains(&row.context) {
                *per_context
                    .entry(row.context.clone())
                    .or_default()
                    .entry(portion.date)
                    .or_default() += ms;
            }
        }
    }
    let counted: Vec<NaiveDate> = reach
        .days()
        .iter()
        .filter(|day| day.end <= now && day_total.get(&day.date).is_some_and(|ms| *ms > 0))
        .map(|day| day.date)
        .collect();
    [
        (SubjectKind::App, per_app),
        (SubjectKind::Context, per_context),
    ]
    .into_iter()
    .flat_map(|(kind, named)| {
        let counted = &counted;
        named.into_iter().map(move |(name, days)| Series {
            subject: Subject { kind, name },
            measure: Measure::Length,
            points: counted
                .iter()
                .map(|date| Point {
                    date: *date,
                    value: minutes(days.get(date).copied().unwrap_or(0)),
                })
                .collect(),
        })
    })
    .collect()
}

/// The names of the [`TOP_SUBJECTS`] longest-used apps, and of the longest-used Contexts that
/// differ from their app's name (those would only repeat the app).
fn busiest(report: &Period, rows: &[ActivityRow]) -> (HashSet<String>, HashSet<String>) {
    let all = totals(report, rows);
    let apps = all
        .apps
        .iter()
        .take(TOP_SUBJECTS)
        .map(|a| a.app.clone())
        .collect();
    let mut contexts: HashMap<&str, i64> = HashMap::new();
    for app in &all.apps {
        for context in app.contexts.iter().filter(|c| c.name != app.app) {
            *contexts.entry(&context.name).or_default() += context.seconds;
        }
    }
    let mut ranked: Vec<(&str, i64)> = contexts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    let names = ranked
        .into_iter()
        .take(TOP_SUBJECTS)
        .map(|(name, _)| name.to_owned())
        .collect();
    (apps, names)
}

// A Day's milliseconds are far inside f64's exact range.
#[allow(clippy::cast_precision_loss)]
fn minutes(millis: i64) -> f64 {
    millis as f64 / 60_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::insights::{Measure, SubjectKind};

    fn row(app: &str, title: &str, day: u32, from: &str, to: &str) -> ActivityRow {
        ActivityRow::new(
            app,
            title,
            format!("2026-05-{day:02}T{from}:00Z").parse().unwrap(),
            format!("2026-05-{day:02}T{to}:00Z").parse().unwrap(),
        )
    }

    fn period() -> Period {
        Period::parse("2026-05-01", "2026-05-31", "UTC").unwrap()
    }

    fn now() -> DateTime<Utc> {
        "2026-06-01T12:00:00Z".parse().unwrap()
    }

    fn find<'a>(all: &'a [Series], kind: SubjectKind, name: &str) -> &'a Series {
        all.iter()
            .find(|s| s.subject.kind == kind && s.subject.name == name)
            .expect("series exists")
    }

    fn minutes(series: &Series) -> Vec<(u32, f64)> {
        let mut found: Vec<_> = series
            .points
            .iter()
            .map(|p| (p.date.format("%d").to_string().parse().unwrap(), p.value))
            .collect();
        found.sort_by_key(|(day, _)| *day);
        found
    }

    #[test]
    fn an_app_is_measured_in_minutes_on_each_day_with_any_activity() {
        let rows = [
            row("Brave", "", 4, "09:00", "10:00"),
            row("Slack", "", 4, "10:00", "10:30"),
            row("Slack", "", 5, "10:00", "10:30"),
        ];
        let all = series(&period(), &period(), &rows, now());
        let brave = find(&all, SubjectKind::App, "Brave");
        assert_eq!(brave.measure, Measure::Length);
        // 5 May has Activity (Slack), so Brave counts 0 there.
        assert_eq!(minutes(brave), [(4, 60.0), (5, 0.0)]);
    }

    #[test]
    fn a_day_with_no_activity_at_all_is_left_out() {
        let rows = [
            row("Brave", "", 4, "09:00", "10:00"),
            row("Brave", "", 6, "09:00", "10:00"),
        ];
        let all = series(&period(), &period(), &rows, now());
        assert_eq!(
            minutes(find(&all, SubjectKind::App, "Brave")),
            [(4, 60.0), (6, 60.0)]
        );
    }

    #[test]
    fn a_day_that_has_not_finished_is_left_out() {
        let rows = [
            row("Brave", "", 4, "09:00", "10:00"),
            row("Brave", "", 5, "09:00", "10:00"),
        ];
        let midday: DateTime<Utc> = "2026-05-05T12:00:00Z".parse().unwrap();
        let all = series(&period(), &period(), &rows, midday);
        assert_eq!(minutes(find(&all, SubjectKind::App, "Brave")), [(4, 60.0)]);
    }

    #[test]
    fn a_context_that_is_not_just_its_app_is_measured_too() {
        let rows = [
            row("Code", "main.rs \u{2014} vitarchy", 4, "09:00", "11:00"),
            row("Slack", "", 4, "11:00", "11:30"),
        ];
        let all = series(&period(), &period(), &rows, now());
        assert_eq!(
            minutes(find(&all, SubjectKind::Context, "vitarchy")),
            [(4, 120.0)]
        );
        // Slack's Context is Slack itself, already an app series.
        assert!(
            !all.iter()
                .any(|s| s.subject.kind == SubjectKind::Context && s.subject.name == "Slack")
        );
    }

    #[test]
    fn only_the_busiest_five_apps_are_measured() {
        let rows: Vec<_> = (0..7)
            .map(|i| {
                row(
                    &format!("App{i}"),
                    "",
                    4,
                    "09:00",
                    &format!("{:02}:00", 10 + i),
                )
            })
            .collect();
        let all = series(&period(), &period(), &rows, now());
        let apps = all
            .iter()
            .filter(|s| s.subject.kind == SubjectKind::App)
            .count();
        assert_eq!(apps, TOP_SUBJECTS);
        assert!(!all.iter().any(|s| s.subject.name == "App0"));
    }
}
