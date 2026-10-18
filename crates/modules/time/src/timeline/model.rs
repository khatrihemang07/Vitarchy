//! The shapes the Timeline is built from and into.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::activity::Source;

/// A stretch of time, `from` up to `to`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeRange {
    /// Where it begins.
    pub from: DateTime<Utc>,
    /// Where it ends.
    pub to: DateTime<Utc>,
}

impl TimeRange {
    /// Every moment the database can store; rebuilding it rebuilds the whole Timeline.
    #[must_use]
    pub fn everything() -> Self {
        // 0001-01-01 and 9999-12-31, the bounds Postgres accepts for sane timestamps.
        let bound = |secs| DateTime::from_timestamp(secs, 0).unwrap_or_default();
        Self {
            from: bound(-62_135_596_800),
            to: bound(253_402_300_799),
        }
    }

    /// This range stretched to also cover `other`.
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self {
            from: self.from.min(other.from),
            to: self.to.max(other.to),
        }
    }
}

/// What one Source said about one stretch of time, normalised so Sources can be compared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// Who said it.
    pub source: Source,
    /// When the stretch began.
    pub start_at: DateTime<Utc>,
    /// When it ended.
    pub end_at: DateTime<Utc>,
    /// The frontmost app.
    pub app: String,
    /// The window title.
    pub title: String,
    /// The page address, when the Source records one.
    pub url: Option<String>,
    /// Whether the person was away; `None` when this Source cannot tell.
    pub away: Option<bool>,
}

/// Whether the person was away during a Segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Away {
    /// Some Source says the person was away.
    Yes,
    /// Some Source says the person was there.
    No,
    /// No Source covering it can tell.
    Unknown,
}

impl Away {
    /// The word stored in the database; equals the serialized name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Yes => "yes",
            Self::No => "no",
            Self::Unknown => "unknown",
        }
    }

    /// The Away stored as `word` by [`Away::as_str`]; `None` for any other word.
    #[must_use]
    pub fn parse(word: &str) -> Option<Self> {
        [Self::Yes, Self::No, Self::Unknown]
            .into_iter()
            .find(|away| away.as_str() == word)
    }
}

/// One stretch of the Timeline in a single app and window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    /// When it began.
    pub start_at: DateTime<Utc>,
    /// When it ended.
    pub end_at: DateTime<Utc>,
    /// The frontmost app.
    pub app: String,
    /// The window title.
    pub title: String,
    /// The page address, when a Source that records them covered it.
    pub url: Option<String>,
    /// Whether the person was away.
    pub away: Away,
    /// Every Source that covered it, in `Source::all()` order.
    pub sources: Vec<Source>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn away_is_stored_as_its_serialized_name() {
        for away in [Away::Yes, Away::No, Away::Unknown] {
            assert_eq!(serde_json::to_value(away).unwrap(), away.as_str());
        }
    }

    #[test]
    fn everything_contains_any_realistic_time() {
        let all = TimeRange::everything();
        let now = Utc::now();
        assert!(all.from < now && now < all.to);
    }

    #[test]
    fn away_parses_back_from_its_stored_word_and_rejects_others() {
        for away in [Away::Yes, Away::No, Away::Unknown] {
            assert_eq!(Away::parse(away.as_str()), Some(away));
        }
        assert_eq!(Away::parse("maybe"), None);
        assert_eq!(Away::parse("Yes"), None);
    }

    #[test]
    fn a_union_stretches_to_the_earliest_start_and_latest_end() {
        let at = |s| DateTime::from_timestamp(s, 0).unwrap();
        let range = |from, to| TimeRange {
            from: at(from),
            to: at(to),
        };
        assert_eq!(range(10, 20).union(range(15, 30)), range(10, 30));
        assert_eq!(range(10, 20).union(range(0, 5)), range(0, 20));
        assert_eq!(range(10, 20).union(range(12, 14)), range(10, 20));
    }
}
