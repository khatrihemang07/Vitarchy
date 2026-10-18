//! When no keyboard or mouse use counts as Away (ADR 0011): a threshold, and an exception
//! for watching.

use chrono::Duration;

/// How long a no-input stretch must last to count as Away. This is the one value to change:
/// edit it here and the Timeline rebuilds itself once, the next time it is read (the rules
/// fingerprint no longer matches). An hour is the default because below it the person is
/// reading, thinking or on a call; measured on real Snapshots, an hour separates those
/// pauses from leaving the computer.
pub const AWAY_AFTER: Duration = Duration::minutes(60);

/// How much no-input time in a watching place stays Active before the rest can be Away.
/// Two hours covers a film or a long call.
pub const WATCHING_CAP: Duration = Duration::hours(2);

/// The rules a Source applies to its no-input time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AwayRules {
    /// No-input stretches shorter than this are Active.
    pub away_after: Duration,
    /// Watching places keep this much no-input time Active.
    pub watching_cap: Duration,
}

impl AwayRules {
    /// The rules in force.
    #[must_use]
    pub const fn current() -> Self {
        Self {
            away_after: AWAY_AFTER,
            watching_cap: WATCHING_CAP,
        }
    }

    /// The current rules with another threshold.
    #[must_use]
    pub const fn with_away_after(self, away_after: Duration) -> Self {
        Self { away_after, ..self }
    }
}

/// How much of one no-input stretch is Away: its `no_input` time, less what watching keeps
/// Active (the `watching` part, up to the cap), when what remains lasts at least
/// `rules.away_after`. Away starts at the last input, so it is the end of the stretch.
#[must_use]
pub fn stretch_away(no_input: Duration, watching: Duration, rules: &AwayRules) -> Duration {
    let tail = no_input - watching.min(rules.watching_cap);
    if tail > Duration::zero() && tail >= rules.away_after {
        tail
    } else {
        Duration::zero()
    }
}
