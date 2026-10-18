//! Cutting Segments to a Period and to its Days.

use chrono::{DateTime, NaiveDate, Utc};

use super::model::SegmentView;
use crate::{
    activity::Source,
    report::period::{Period, local_midnight},
    timeline::model::{Away, Segment},
};

/// The part of one Segment that falls on one Day of the Period.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    /// The local Day.
    pub date: NaiveDate,
    /// When the piece begins.
    pub start_at: DateTime<Utc>,
    /// When the piece ends.
    pub end_at: DateTime<Utc>,
    /// The frontmost app.
    pub app: String,
    /// The window title.
    pub title: String,
    /// The page address, when one was recorded.
    pub url: Option<String>,
    /// Whether the person was away.
    pub away: Away,
    /// Every Source that covered the Segment.
    pub sources: Vec<Source>,
}

impl Piece {
    /// Its length in whole seconds.
    #[must_use]
    pub fn secs(&self) -> i64 {
        (self.end_at - self.start_at).num_seconds()
    }

    /// True when the person was known to be there.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.away == Away::No
    }

    /// The piece as the API shows it.
    #[must_use]
    pub fn view(&self) -> SegmentView {
        SegmentView {
            start_at: self.start_at,
            end_at: self.end_at,
            app: self.app.clone(),
            title: self.title.clone(),
            url: self.url.clone(),
            away: self.away,
            sources: self.sources.clone(),
        }
    }
}

/// Cuts `segments` (in time order) to the Period's UTC range, then at each local midnight.
/// Day lengths follow the time zone, so a DST Day holds 23 or 25 hours.
#[must_use]
pub fn clip(segments: &[Segment], period: &Period) -> Vec<Piece> {
    let (range_start, range_end) = period.utc_range();
    let mut pieces = Vec::new();
    for segment in segments {
        let mut at = segment.start_at.max(range_start);
        let end = segment.end_at.min(range_end);
        while at < end {
            let date = at.with_timezone(&period.tz).date_naive();
            let midnight = local_midnight(period.tz, date.succ_opt().unwrap_or(date));
            // Guards a zone whose midnight lookup would not move forward.
            let piece_end = if midnight > at {
                midnight.min(end)
            } else {
                end
            };
            pieces.push(Piece {
                date,
                start_at: at,
                end_at: piece_end,
                app: segment.app.clone(),
                title: segment.title.clone(),
                url: segment.url.clone(),
                away: segment.away,
                sources: segment.sources.clone(),
            });
            at = piece_end;
        }
    }
    pieces
}
