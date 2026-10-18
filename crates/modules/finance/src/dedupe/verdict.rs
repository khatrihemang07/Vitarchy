//! Whether two rows of one Account are the same Transaction, and by which evidence. The
//! rungs run strongest first; a rung present on both rows decides, so a disagreement means
//! "not the same" and never falls through to a weaker rung.

use super::evidence::Evidence;

/// Descriptions this similar (or more) count as the same text.
const SIMILARITY_THRESHOLD: f64 = 0.6;
/// A description that is a prefix of the other counts only when at least this long.
const MIN_PREFIX_CHARS: usize = 8;
/// Rows this many days apart may still match when an id or balance decides.
const DATE_WINDOW_DAYS: i64 = 1;

/// The evidence that decided a match, strongest first (the derived order is the pairing order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rung {
    /// Both rows carry the issuer's id and they are equal.
    SourceId,
    /// Both rows carry a balance after and they are equal.
    Balance,
    /// Both rows carry a reference and they are equal.
    Reference,
    /// The descriptions are alike.
    Description,
}

/// A match: the deciding rung, the days apart and how alike the descriptions are
/// (per mille), which later break ties between competing pairs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verdict {
    /// What decided it.
    pub rung: Rung,
    /// Days between the two dates.
    pub days_apart: i64,
    /// Description similarity per mille (0 when not compared).
    pub closeness: u32,
}

/// `Some` when the rows are the same Transaction.
#[must_use]
pub fn verdict(a: &Evidence, b: &Evidence) -> Option<Verdict> {
    if a.amount_paise != b.amount_paise || a.direction != b.direction {
        return None;
    }
    let days_apart = (a.date - b.date).num_days().abs();
    if days_apart > DATE_WINDOW_DAYS {
        return None;
    }
    let decided = |rung, closeness| {
        Some(Verdict {
            rung,
            days_apart,
            closeness,
        })
    };
    if let (Some(x), Some(y)) = (&a.source_id, &b.source_id) {
        return if x == y {
            decided(Rung::SourceId, 0)
        } else {
            None
        };
    }
    if let (Some(x), Some(y)) = (a.balance_paise, b.balance_paise) {
        return if x == y {
            decided(Rung::Balance, 0)
        } else {
            None
        };
    }
    // Only an id or a balance may bridge a day; weaker evidence needs the same date.
    if days_apart != 0 {
        return None;
    }
    if let (Some(x), Some(y)) = (&a.reference, &b.reference) {
        return if x == y {
            decided(Rung::Reference, 0)
        } else {
            None
        };
    }
    let closeness = description_closeness(&a.description, &b.description)?;
    decided(Rung::Description, closeness)
}

/// How alike two normalised descriptions are (per mille), or `None` when they are not alike
/// enough: equal, one a long-enough prefix of the other, or similar by Sørensen-Dice.
fn description_closeness(a: &str, b: &str) -> Option<u32> {
    if a == b {
        return Some(1000);
    }
    let (short, long) = if a.len() <= b.len() { (a, b) } else { (b, a) };
    if short.chars().count() >= MIN_PREFIX_CHARS && long.starts_with(short) {
        return Some(1000);
    }
    // Sørensen-Dice over bigrams tracks Python's difflib ratio closely on bank narrations
    // (normalised Levenshtein ran 0.05-0.15 lower on the same samples), and truncated or
    // re-wrapped text keeps most of its bigrams.
    let similarity = strsim::sorensen_dice(a, b);
    (similarity >= SIMILARITY_THRESHOLD).then(|| per_mille(similarity))
}

/// A ratio in 0..=1 as whole per mille.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to 0..=1000
fn per_mille(ratio: f64) -> u32 {
    (ratio.clamp(0.0, 1.0) * 1000.0).round() as u32
}
