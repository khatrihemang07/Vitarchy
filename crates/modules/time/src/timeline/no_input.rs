//! No-input stretches: Clockify's `idle_secs` keeps counting across rows it splits (at midnight,
//! at a window-title change without input), so Away is decided once per stretch, not per row.

use chrono::{DateTime, Duration, Utc};

use super::{
    away_rules::{AwayRules, stretch_away},
    model::TimeRange,
};
use crate::activity::{Activity, Source};

/// Bump when the way rows are grouped into stretches or Away is assigned to them changes: it is
/// part of the rules fingerprint, so the stored Timeline is rebuilt.
pub const STRETCH_LOGIC_VERSION: u32 = 1;

/// How far apart (in either direction) two rows may be and still be one stretch, and how much
/// shorter the later row's no-input time may fall short of the carried-on amount. Clockify
/// stamps the end of one row and the start of the next a moment apart.
pub const STRETCH_TOLERANCE: Duration = Duration::seconds(2);

/// One Clockify row as the no-input rules see it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoInputRow {
    /// When the row began.
    pub start_at: DateTime<Utc>,
    /// When the row ended.
    pub end_at: DateTime<Utc>,
    /// Seconds without input as Clockify reported them at the row's end; may exceed its length.
    pub no_input_secs: f64,
    /// Whether the row's app or site is a watching place.
    pub watching: bool,
}

impl NoInputRow {
    /// The row of a Clockify Activity.
    #[must_use]
    pub fn of(activity: &Activity) -> Self {
        Self {
            start_at: activity.start_at,
            end_at: activity.end_at,
            no_input_secs: activity.extras.idle_secs.unwrap_or(0.0),
            watching: super::watching::is_watching(&activity.app, activity.extras.url.as_deref()),
        }
    }

    fn length(&self) -> Duration {
        self.end_at - self.start_at
    }

    /// The part of the row without input: at its end, at most the whole row.
    fn no_input(&self) -> Duration {
        reported(self.no_input_secs).min(self.length())
    }

    /// Whether `next` carries on the no-input time this row ends with.
    fn continues_into(&self, next: &Self) -> bool {
        let touching = (next.start_at - self.end_at).abs() <= STRETCH_TOLERANCE;
        let carried = reported(self.no_input_secs) + self.length();
        touching && reported(next.no_input_secs) >= carried - STRETCH_TOLERANCE
    }
}

/// Clockify's seconds as a duration; negative or absurdly large values are bounded first so
/// the conversion cannot overflow.
#[allow(clippy::cast_possible_truncation)]
fn reported(secs: f64) -> Duration {
    Duration::milliseconds((secs.clamp(0.0, 1.0e9) * 1000.0).round() as i64)
}

/// Whether `later` carries on the no-input time of `earlier`, for the rebuild to know which
/// neighbouring rows a changed row drags in.
#[must_use]
pub fn continues(earlier: &Activity, later: &Activity) -> bool {
    NoInputRow::of(earlier).continues_into(&NoInputRow::of(later))
}

/// `range` stretched over the Clockify rows in `near` that carry on a no-input stretch with a
/// row inside it, so a rebuild of `range` also redoes the rows whose Away it changes. `near`
/// holds the rows within [`STRETCH_TOLERANCE`] of `range`; the caller repeats until it settles.
#[must_use]
pub fn reach(range: TimeRange, near: &[Activity]) -> TimeRange {
    let clockify = || near.iter().filter(|a| a.source == Source::Clockify);
    let inside = |a: &&Activity| a.start_at < range.to && a.end_at > range.from;
    clockify()
        .filter(|a| !inside(a))
        .filter(|outside| {
            clockify()
                .filter(inside)
                .any(|i| continues(outside, i) || continues(i, outside))
        })
        .fold(range, |all, a| {
            all.union(TimeRange {
                from: a.start_at,
                to: a.end_at,
            })
        })
}

/// The Away part, at the end of each row, in the order `rows` came in. Rows that carry on each
/// other's no-input time form a stretch; each stretch is judged once and its Away time is
/// handed to its rows from the end backwards, so Away starts at the last input.
#[must_use]
pub fn away_tails(rows: &[NoInputRow], rules: &AwayRules) -> Vec<Duration> {
    let mut order: Vec<usize> = (0..rows.len()).collect();
    order.sort_by_key(|&i| rows[i].start_at);
    let mut tails = vec![Duration::zero(); rows.len()];
    for stretch in stretches(rows, &order) {
        let parts = stretch.iter().map(|&i| &rows[i]);
        let no_input = parts.clone().map(NoInputRow::no_input).sum();
        let watching = parts.filter(|r| r.watching).map(NoInputRow::no_input).sum();
        let mut left = stretch_away(no_input, watching, rules);
        for &i in stretch.iter().rev() {
            let take = left.min(rows[i].no_input());
            tails[i] = take;
            left -= take;
        }
    }
    tails
}

/// `order` (indices into `rows`, by start) split wherever a row does not carry on the last.
fn stretches(rows: &[NoInputRow], order: &[usize]) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = Vec::new();
    for &i in order {
        let carries_on = out
            .last()
            .and_then(|open| open.last())
            .is_some_and(|&previous| rows[previous].continues_into(&rows[i]));
        if let (true, Some(open)) = (carries_on, out.last_mut()) {
            open.push(i);
        } else {
            out.push(vec![i]);
        }
    }
    out
}
