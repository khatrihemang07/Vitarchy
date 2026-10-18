//! Builders for the unit tests of the Timeline analysis.

use chrono::{DateTime, TimeZone, Utc};

use super::clip::Piece;
use crate::timeline::model::Away;

/// A moment `minute` minutes after 2026-10-17T00:00Z, plus `second` seconds.
pub fn at(minute: i64, second: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 10, 17, 0, 0, 0).unwrap()
        + chrono::Duration::seconds(minute * 60 + second)
}

/// A piece from second `from` to second `to` (after the base moment) in `app`.
pub fn piece_s(app: &str, from: i64, to: i64, away: Away) -> Piece {
    let base = at(0, 0);
    let start_at = base + chrono::Duration::seconds(from);
    Piece {
        date: start_at.date_naive(),
        start_at,
        end_at: base + chrono::Duration::seconds(to),
        app: app.to_owned(),
        title: String::new(),
        url: None,
        away,
        sources: vec![],
    }
}

/// An Active piece from second `from` to second `to` in `app`.
pub fn active_s(app: &str, from: i64, to: i64) -> Piece {
    piece_s(app, from, to, Away::No)
}
