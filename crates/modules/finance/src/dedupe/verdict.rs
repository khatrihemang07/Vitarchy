//! Whether two rows of one Account are the same Transaction, and by which evidence. The
//! rungs run strongest first; a rung present on both rows decides, so a disagreement means
//! "not the same" and never falls through to a weaker rung. A final rung, a unique
//! partner between a payment app and an issuer, is decided over whole sets in `pairing`.

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

/// Days apart when the rows have an equal amount and direction and dates within the window:
/// the ground every rung stands on.
fn days_apart_if_close(a: &Evidence, b: &Evidence) -> Option<i64> {
    if a.amount_paise != b.amount_paise || a.direction != b.direction {
        return None;
    }
    let days_apart = (a.date - b.date).num_days().abs();
    (days_apart <= DATE_WINDOW_DAYS).then_some(days_apart)
}

/// Whether both rows carry a value and the values differ: a rung present on both sides that
/// disagrees means "not the same".
fn disagree<T: PartialEq>(a: Option<&T>, b: Option<&T>) -> bool {
    matches!((a, b), (Some(x), Some(y)) if x != y)
}

/// Whether the rows could be the same payment by amount, direction, date and origin alone:
/// one is only a payment app's and the other only an issuer's, an equal amount and
/// direction, within the date window, and no reference or shared-namespace id that
/// disagrees. The unique-partner rung decides when this holds for exactly one pair.
#[must_use]
pub fn unique_partner_possible(a: &Evidence, b: &Evidence) -> bool {
    days_apart_if_close(a, b).is_some()
        && a.origins.complement(b.origins)
        && !disagree(a.reference.as_ref(), b.reference.as_ref())
        && !(a.origins.share_ids(b.origins) && disagree(a.source_id.as_ref(), b.source_id.as_ref()))
}

/// `Some` when the rows are the same Transaction.
#[must_use]
pub fn verdict(a: &Evidence, b: &Evidence) -> Option<Verdict> {
    let days_apart = days_apart_if_close(a, b)?;
    let decided = |rung, closeness| {
        Some(Verdict {
            rung,
            days_apart,
            closeness,
        })
    };
    if let (Some(x), Some(y)) = (&a.source_id, &b.source_id)
        && a.origins.share_ids(b.origins)
    {
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
    // A reference bridges a day as an id and a balance do; descriptions need the same date.
    if let (Some(x), Some(y)) = (&a.reference, &b.reference) {
        return if x == y {
            decided(Rung::Reference, 0)
        } else {
            None
        };
    }
    if days_apart != 0 {
        return None;
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
