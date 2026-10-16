//! Turns the spans a subject happened in into its start, end and length series.

use chrono::{DateTime, NaiveDate, Utc};
use chrono_tz::Tz;

use crate::analysis::{
    insights::{Measure, Point, Series, Subject},
    lifelog::minute_of_day,
};

/// How one measure is read off a span.
type ValueOf = fn(Tz, &Span) -> f64;

/// One occurrence of a subject: the local Day it belongs to and when it began and ended.
pub(super) struct Span {
    pub date: NaiveDate,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Start, end and length series of `subject` (nothing when it never happened).
pub(super) fn series_of(subject: &Subject, tz: Tz, spans: &[Span]) -> Vec<Series> {
    if spans.is_empty() {
        return Vec::new();
    }
    let measures: [(Measure, ValueOf); 3] = [
        (Measure::Start, |tz, s| minute_of_day(tz, s.start)),
        (Measure::End, |tz, s| minute_of_day(tz, s.end)),
        (Measure::Length, |_, s| minutes(s)),
    ];
    measures
        .into_iter()
        .map(|(measure, value)| Series {
            subject: subject.clone(),
            measure,
            points: spans
                .iter()
                .map(|s| Point {
                    date: s.date,
                    value: value(tz, s),
                })
                .collect(),
        })
        .collect()
}

/// A span's length in minutes.
// Spans are far inside f64's exact range.
#[allow(clippy::cast_precision_loss)]
pub(super) fn minutes(span: &Span) -> f64 {
    (span.end - span.start).num_seconds() as f64 / 60.0
}
