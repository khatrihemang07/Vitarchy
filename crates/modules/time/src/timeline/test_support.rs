//! Builders shared by the Timeline unit tests.

use chrono::{DateTime, Utc};

use super::model::Observation;
use crate::activity::{Source, Strengths};

/// A time `secs` after an arbitrary fixed origin.
pub fn at(secs: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(1_000_000 + secs, 0).unwrap()
}

/// An observation by `source` from second `start` to `end`, in `app` with an empty title.
pub fn obs(source: Source, start: i64, end: i64, app: &str, away: Option<bool>) -> Observation {
    Observation {
        source,
        start_at: at(start),
        end_at: at(end),
        app: app.to_owned(),
        title: String::new(),
        url: None,
        away,
    }
}

/// The strengths the real Sources declare, looked up from their descriptors.
pub fn real(source: Source) -> Strengths {
    source.descriptor().strengths
}
