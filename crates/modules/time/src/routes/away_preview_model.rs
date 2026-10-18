//! The body of `GET /api/time/timeline/away-preview`: the Away rules in force and what each
//! threshold would give. Every rule shown comes from the code that applies it.

use serde::Serialize;
use utoipa::ToSchema;

use crate::timeline::{
    AwayRules,
    preview::{PREVIEW_DAYS, ThresholdEffect},
    watching::{WATCHING_APPS, WATCHING_SITES},
};

/// The current rules and, for each choice of threshold, what Away and Active would be.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct AwayPreview {
    /// The threshold in force, in seconds.
    pub current_secs: i64,
    /// How much no-input time watching keeps Active, in seconds.
    pub watching_cap_secs: i64,
    /// Apps where no input is watching or a call.
    pub watching_apps: Vec<String>,
    /// Sites (with their subdomains) where no input is watching or a call.
    pub watching_sites: Vec<String>,
    /// How many Days back the choices look.
    pub days: i64,
    /// One row per threshold, lowest first.
    pub choices: Vec<PreviewChoice>,
}

/// Clockify's Away and Active time per Day under one threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct PreviewChoice {
    /// The threshold, in seconds.
    pub away_after_secs: i64,
    /// Average Away seconds per Day that has Clockify Activity.
    pub away_secs_per_day: i64,
    /// Average Active seconds per Day that has Clockify Activity.
    pub active_secs_per_day: i64,
}

impl AwayPreview {
    /// The rules in force with `effects` as the choices.
    #[must_use]
    pub fn new(effects: &[ThresholdEffect]) -> Self {
        let rules = AwayRules::current();
        Self {
            current_secs: rules.away_after.num_seconds(),
            watching_cap_secs: rules.watching_cap.num_seconds(),
            watching_apps: WATCHING_APPS.map(str::to_owned).to_vec(),
            watching_sites: WATCHING_SITES.map(str::to_owned).to_vec(),
            days: PREVIEW_DAYS,
            choices: effects
                .iter()
                .map(|e| PreviewChoice {
                    away_after_secs: e.away_after_secs,
                    away_secs_per_day: e.away_secs_per_day,
                    active_secs_per_day: e.active_secs_per_day,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_carries_the_rules_the_timeline_applies() {
        let preview = AwayPreview::new(&[]);
        assert_eq!(
            (preview.current_secs, preview.watching_cap_secs),
            (3600, 7200)
        );
        assert!(preview.watching_apps.iter().any(|a| a == "VLC"));
        assert!(preview.watching_sites.iter().any(|s| s == "youtube.com"));
        assert_eq!(preview.days, PREVIEW_DAYS);
    }
}
