//! A text fingerprint of everything that decides stored Away, so the Timeline is rebuilt
//! whenever any of it changes and forgetting a bump is impossible.

use super::{
    away_rules::AwayRules,
    no_input::{STRETCH_LOGIC_VERSION, STRETCH_TOLERANCE},
    watching::{WATCHING_APPS, WATCHING_SITES},
};

/// The fingerprint of the rules in force.
#[must_use]
pub fn current() -> String {
    fingerprint(
        &AwayRules::current(),
        &WATCHING_APPS,
        &WATCHING_SITES,
        STRETCH_LOGIC_VERSION,
    )
}

/// The fingerprint of a set of rule values. Readable on purpose (it is stored and logged), and
/// independent of the order and case of the watching lists.
#[must_use]
pub fn fingerprint(rules: &AwayRules, apps: &[&str], sites: &[&str], stretch_logic: u32) -> String {
    format!(
        "stretch={stretch_logic};tolerance={}s;away_after={}s;cap={}s;apps={};sites={}",
        STRETCH_TOLERANCE.num_seconds(),
        rules.away_after.num_seconds(),
        rules.watching_cap.num_seconds(),
        canonical(apps),
        canonical(sites),
    )
}

fn canonical(names: &[&str]) -> String {
    let mut names: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
    names.sort();
    names.join(",")
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn base() -> String {
        fingerprint(&AwayRules::current(), &["VLC", "Zoom"], &["a.com"], 1)
    }

    #[test]
    fn the_same_rules_give_the_same_fingerprint_in_any_list_order() {
        let shuffled = fingerprint(&AwayRules::current(), &["zoom", "vlc"], &["a.com"], 1);
        assert_eq!(base(), shuffled);
    }

    #[test]
    fn changing_any_value_changes_it() {
        let rules = AwayRules::current();
        let changed = [
            fingerprint(
                &rules.with_away_after(Duration::minutes(30)),
                &["VLC", "Zoom"],
                &["a.com"],
                1,
            ),
            fingerprint(
                &AwayRules {
                    watching_cap: Duration::hours(3),
                    ..rules
                },
                &["VLC", "Zoom"],
                &["a.com"],
                1,
            ),
            fingerprint(&rules, &["VLC"], &["a.com"], 1),
            fingerprint(&rules, &["VLC", "Zoom"], &["a.com", "b.com"], 1),
            fingerprint(&rules, &["VLC", "Zoom"], &["a.com"], 2),
        ];
        for other in changed {
            assert_ne!(base(), other);
        }
    }

    #[test]
    fn the_fingerprint_of_the_rules_in_force_follows_the_code_constants() {
        let text = current();
        assert!(text.contains("away_after=3600s;cap=7200s"), "{text}");
        assert!(text.contains("youtube.com"), "{text}");
        assert!(!text.contains("spotify"), "{text}");
    }
}
