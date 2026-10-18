//! What each Away threshold would have made of the stored Clockify Activity: the numbers
//! behind the Sources "Away rules" card. Pure; the caller reads the Activity.

use std::collections::BTreeSet;

use chrono::Duration;

use super::{
    away_rules::{AwayRules, PREVIEW_CHOICES},
    no_input::{NoInputRow, away_tails},
};
use crate::activity::{Activity, Source};

/// How many Days back the preview looks. The caller reads Activity for this window and the
/// card names it, so it is stated once.
pub const PREVIEW_DAYS: i64 = 28;

/// Clockify's Away and Active time per Day under one threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThresholdEffect {
    /// The threshold, in seconds.
    pub away_after_secs: i64,
    /// Average Away seconds per Day that has Clockify Activity.
    pub away_secs_per_day: i64,
    /// Average Active seconds per Day that has Clockify Activity.
    pub active_secs_per_day: i64,
}

/// Runs the Away rules over the Clockify rows of `activities` for every preview choice, lowest
/// first. Per-Day averages divide by the number of UTC days those rows start on (at least one):
/// the preview has no time zone, and an estimate is enough to compare thresholds.
#[must_use]
pub fn preview(activities: &[Activity]) -> Vec<ThresholdEffect> {
    let clockify: Vec<&Activity> = activities
        .iter()
        .filter(|a| a.source == Source::Clockify)
        .collect();
    let days = clockify
        .iter()
        .map(|a| a.start_at.date_naive())
        .collect::<BTreeSet<_>>()
        .len()
        .max(1);
    let rows: Vec<NoInputRow> = clockify.iter().map(|a| NoInputRow::of(a)).collect();
    let current = AwayRules::current();
    PREVIEW_CHOICES
        .iter()
        .map(|&minutes| {
            effect(
                &rows,
                days,
                current.with_away_after(Duration::minutes(minutes)),
            )
        })
        .collect()
}

fn effect(rows: &[NoInputRow], days: usize, rules: AwayRules) -> ThresholdEffect {
    let away: i64 = away_tails(rows, &rules)
        .iter()
        .map(Duration::num_seconds)
        .sum();
    let total: i64 = rows
        .iter()
        .map(|r| (r.end_at - r.start_at).num_seconds())
        .sum();
    let days = i64::try_from(days).unwrap_or(i64::MAX);
    ThresholdEffect {
        away_after_secs: rules.away_after.num_seconds(),
        away_secs_per_day: away / days,
        active_secs_per_day: (total - away) / days,
    }
}
