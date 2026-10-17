//! The busiest Contexts of a Period, ranked once on the server.

use std::collections::HashMap;

use serde::Serialize;
use utoipa::ToSchema;

use super::Totals;

/// Apps, and Contexts, listed as the busiest: the Overview and the Insights both use it.
pub const TOP_SUBJECTS: usize = 5;

/// A Context and the seconds spent in it, across apps.
#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct RankedContext {
    /// The Context's name.
    pub name: String,
    /// Seconds in it, summed over every app it appears under.
    pub seconds: i64,
}

/// The `limit` longest-used Contexts, longest first (names break ties). A Context that only
/// repeats its app's name (any app that is neither an editor nor a browser) is left out, since
/// the apps already show it.
#[must_use]
pub fn top_contexts(totals: &Totals, limit: usize) -> Vec<RankedContext> {
    let mut by_name: HashMap<&str, i64> = HashMap::new();
    for app in &totals.apps {
        for context in app.contexts.iter().filter(|c| c.name != app.app) {
            *by_name.entry(&context.name).or_default() += context.seconds;
        }
    }
    let mut ranked: Vec<RankedContext> = by_name
        .into_iter()
        .map(|(name, seconds)| RankedContext {
            name: name.to_owned(),
            seconds,
        })
        .collect();
    ranked.sort_by(|a, b| b.seconds.cmp(&a.seconds).then_with(|| a.name.cmp(&b.name)));
    ranked.truncate(limit);
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::activity::{AppTotal, ContextTotal};

    fn context(name: &str, seconds: i64) -> ContextTotal {
        ContextTotal {
            name: name.into(),
            seconds,
            titles: Vec::new(),
        }
    }

    fn app(name: &str, contexts: Vec<ContextTotal>) -> AppTotal {
        AppTotal {
            app: name.into(),
            seconds: contexts.iter().map(|c| c.seconds).sum(),
            contexts,
        }
    }

    fn totals(apps: Vec<AppTotal>) -> Totals {
        Totals {
            total_seconds: apps.iter().map(|a| a.seconds).sum(),
            apps,
        }
    }

    fn names(ranked: &[RankedContext]) -> Vec<&str> {
        ranked.iter().map(|c| c.name.as_str()).collect()
    }

    #[test]
    fn the_longest_contexts_come_first_without_those_that_only_repeat_their_app() {
        let all = totals(vec![
            app(
                "Brave",
                vec![context("YouTube", 5000), context("Other pages", 4000)],
            ),
            app(
                "Code",
                vec![context("vitarchy", 6000), context("notes", 2000)],
            ),
            app("Slack", vec![context("Slack", 3000)]),
        ]);
        assert_eq!(
            names(&top_contexts(&all, 3)),
            ["vitarchy", "YouTube", "Other pages"]
        );
    }

    #[test]
    fn a_context_under_two_apps_is_added_up() {
        let all = totals(vec![
            app("Brave", vec![context("Docs", 100)]),
            app("Firefox", vec![context("Docs", 250)]),
        ]);
        assert_eq!(
            top_contexts(&all, 5),
            [RankedContext {
                name: "Docs".into(),
                seconds: 350
            }]
        );
    }

    #[test]
    fn equal_lengths_are_ordered_by_name() {
        let all = totals(vec![app("Code", vec![context("b", 10), context("a", 10)])]);
        assert_eq!(names(&top_contexts(&all, 5)), ["a", "b"]);
    }
}
