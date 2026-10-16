//! Where screen time goes: per app, per Context within it, then the top titles.

use std::collections::HashMap;

use serde::Serialize;
use utoipa::ToSchema;

use super::{ActivityRow, row::millis, seconds};
use crate::analysis::{Period, split_by_day};

/// Titles kept per Context.
pub const TOP_TITLES: usize = 8;

/// Time on one window title.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct TitleTotal {
    /// The title as stored.
    pub title: String,
    /// Seconds.
    pub seconds: i64,
}

/// Time in one Context of an app.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct ContextTotal {
    /// The Context, e.g. a workspace or a site.
    pub name: String,
    /// Seconds, including untitled time.
    pub seconds: i64,
    /// The longest titles, longest first; at most [`TOP_TITLES`].
    pub titles: Vec<TitleTotal>,
}

/// Time in one app.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct AppTotal {
    /// The app name.
    pub app: String,
    /// Seconds.
    pub seconds: i64,
    /// Contexts, longest first.
    pub contexts: Vec<ContextTotal>,
}

/// Screen time over a Period.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct Totals {
    /// Seconds across all apps.
    pub total_seconds: i64,
    /// Apps, longest first.
    pub apps: Vec<AppTotal>,
}

#[derive(Default)]
struct ContextTime {
    millis: i64,
    titles: HashMap<String, i64>,
}

/// `map[key]`, created if absent, without allocating the key when it exists.
pub(super) fn slot<'a, V: Default>(map: &'a mut HashMap<String, V>, key: &str) -> &'a mut V {
    if !map.contains_key(key) {
        map.insert(key.to_owned(), V::default());
    }
    map.get_mut(key).expect("inserted just above")
}

/// Longest first; names break ties so output is stable.
fn by_length<T>(items: &mut [T], key: impl Fn(&T) -> (i64, &str)) {
    items.sort_by(|a, b| {
        let ((sa, na), (sb, nb)) = (key(a), key(b));
        sb.cmp(&sa).then_with(|| na.cmp(nb))
    });
}

/// Totals of the rows' portions inside the Period.
#[must_use]
pub fn totals(period: &Period, rows: &[ActivityRow]) -> Totals {
    let mut apps: HashMap<String, HashMap<String, ContextTime>> = HashMap::new();
    for row in rows {
        let spent: i64 = split_by_day(period, row.start, row.end)
            .iter()
            .map(millis)
            .sum();
        if spent == 0 {
            continue;
        }
        let context = slot(slot(&mut apps, &row.app), &row.context);
        context.millis += spent;
        if !row.title.is_empty() {
            *slot(&mut context.titles, &row.title) += spent;
        }
    }
    let mut apps: Vec<AppTotal> = apps
        .into_iter()
        .map(|(app, contexts)| app_total(app, contexts))
        .collect();
    by_length(&mut apps, |a| (a.seconds, &a.app));
    Totals {
        total_seconds: apps.iter().map(|a| a.seconds).sum(),
        apps,
    }
}

fn app_total(app: String, contexts: HashMap<String, ContextTime>) -> AppTotal {
    let mut contexts: Vec<ContextTotal> = contexts
        .into_iter()
        .map(|(name, time)| context_total(name, time))
        .collect();
    by_length(&mut contexts, |c| (c.seconds, &c.name));
    AppTotal {
        app,
        seconds: contexts.iter().map(|c| c.seconds).sum(),
        contexts,
    }
}

fn context_total(name: String, time: ContextTime) -> ContextTotal {
    let mut titles: Vec<TitleTotal> = time
        .titles
        .into_iter()
        .map(|(title, millis)| TitleTotal {
            title,
            seconds: seconds(millis),
        })
        .collect();
    by_length(&mut titles, |t| (t.seconds, &t.title));
    titles.truncate(TOP_TITLES);
    ContextTotal {
        name,
        seconds: seconds(time.millis),
        titles,
    }
}

#[cfg(test)]
mod tests {
    use super::super::row::at;
    use super::*;

    fn period() -> Period {
        Period::parse("2026-05-01", "2026-05-02", "UTC").unwrap()
    }

    fn row(app: &str, title: &str, start: &str, end: &str) -> ActivityRow {
        ActivityRow::new(app, title, at(start), at(end))
    }

    #[test]
    fn no_rows_is_an_empty_total() {
        let result = totals(&period(), &[]);
        assert_eq!((result.total_seconds, result.apps.len()), (0, 0));
    }

    #[test]
    fn apps_contexts_and_titles_are_sorted_longest_first() {
        let rows = [
            row(
                "Spotify",
                "",
                "2026-05-01T08:00:00Z",
                "2026-05-01T08:10:00Z",
            ),
            row(
                "Code",
                "a.rs — vitarchy",
                "2026-05-01T09:00:00Z",
                "2026-05-01T09:30:00Z",
            ),
            row(
                "Code",
                "b.rs — vitarchy",
                "2026-05-01T10:00:00Z",
                "2026-05-01T10:50:00Z",
            ),
            row(
                "Code",
                "notes — journal",
                "2026-05-01T11:00:00Z",
                "2026-05-01T11:05:00Z",
            ),
        ];
        let result = totals(&period(), &rows);
        let apps: Vec<(&str, i64)> = result
            .apps
            .iter()
            .map(|a| (a.app.as_str(), a.seconds))
            .collect();
        assert_eq!(apps, [("Code", 85 * 60), ("Spotify", 10 * 60)]);
        assert_eq!(result.total_seconds, 95 * 60);
        let contexts: Vec<&str> = result.apps[0]
            .contexts
            .iter()
            .map(|c| c.name.as_str())
            .collect();
        assert_eq!(contexts, ["vitarchy", "journal"]);
        let titles: Vec<&str> = result.apps[0].contexts[0]
            .titles
            .iter()
            .map(|t| t.title.as_str())
            .collect();
        assert_eq!(titles, ["b.rs — vitarchy", "a.rs — vitarchy"]);
    }

    #[test]
    fn untitled_time_counts_toward_the_context_but_lists_no_title() {
        let rows = [row(
            "Finder",
            "",
            "2026-05-01T08:00:00Z",
            "2026-05-01T08:01:00Z",
        )];
        let result = totals(&period(), &rows);
        assert_eq!(result.apps[0].contexts[0].seconds, 60);
        assert!(result.apps[0].contexts[0].titles.is_empty());
    }

    #[test]
    fn the_same_title_in_separate_rows_adds_up() {
        let rows = [
            row(
                "Brave Browser",
                "Lofi - YouTube",
                "2026-05-01T08:00:00Z",
                "2026-05-01T08:01:00Z",
            ),
            row(
                "Brave Browser",
                "Lofi - YouTube",
                "2026-05-01T09:00:00Z",
                "2026-05-01T09:02:00Z",
            ),
        ];
        let context = &totals(&period(), &rows).apps[0].contexts[0];
        assert_eq!(
            (context.name.as_str(), context.titles[0].seconds),
            ("YouTube", 180)
        );
    }

    #[test]
    fn only_the_top_titles_are_kept() {
        let rows: Vec<ActivityRow> = (0..12)
            .map(|i| {
                row(
                    "Code",
                    &format!("f{i} — p"),
                    "2026-05-01T08:00:00Z",
                    &format!("2026-05-01T08:{:02}:00Z", i + 1),
                )
            })
            .collect();
        let context = &totals(&period(), &rows).apps[0].contexts[0];
        assert_eq!(context.titles.len(), TOP_TITLES);
        assert_eq!(context.titles[0].title, "f11 — p");
        assert_eq!(context.seconds, 78 * 60);
    }

    #[test]
    fn a_row_over_midnight_counts_only_the_part_inside_the_period() {
        let rows = [row(
            "Spotify",
            "",
            "2026-04-30T22:00:00Z",
            "2026-05-01T02:00:00Z",
        )];
        assert_eq!(totals(&period(), &rows).total_seconds, 2 * 3600);
    }

    #[test]
    fn a_111_hour_row_is_counted_raw_up_to_the_periods_end() {
        let rows = [row(
            "Spotify",
            "",
            "2026-05-01T20:00:00Z",
            "2026-05-06T11:00:00Z",
        )];
        // 4 h left on day one plus the whole of day two.
        assert_eq!(totals(&period(), &rows).total_seconds, 28 * 3600);
    }

    #[test]
    fn invisible_marks_do_not_split_an_app_in_two() {
        let rows = [
            row(
                "\u{200E}WhatsApp",
                "",
                "2026-05-01T08:00:00Z",
                "2026-05-01T08:01:00Z",
            ),
            row(
                "WhatsApp",
                "",
                "2026-05-01T09:00:00Z",
                "2026-05-01T09:01:00Z",
            ),
        ];
        let result = totals(&period(), &rows);
        assert_eq!((result.apps.len(), result.apps[0].seconds), (1, 120));
    }

    #[test]
    fn fractions_of_a_second_are_summed_before_rounding() {
        let rows = [
            row("A", "", "2026-05-01T08:00:00.400Z", "2026-05-01T08:00:01Z"),
            row("A", "", "2026-05-01T09:00:00.400Z", "2026-05-01T09:00:01Z"),
        ];
        // Two 0.6 s rows are 1.2 s, not 2 s (rounding each) or 0 s (truncating each).
        assert_eq!(totals(&period(), &rows).total_seconds, 1);
    }
}
