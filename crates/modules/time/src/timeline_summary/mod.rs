//! Timeline summary: the pure analysis behind `GET /api/time/timeline`. Segments and
//! entries come in already read; nothing here touches storage.

mod apps;
#[cfg(test)]
mod apps_tests;
mod buckets;
mod clip;
#[cfg(test)]
mod clip_tests;
mod day_shape;
mod domain;
mod figures;
mod focus;
#[cfg(test)]
mod focus_tests;
mod lane;
pub mod model;
mod switches;
mod usual;

#[cfg(test)]
mod summary_tests;
#[cfg(test)]
mod test_support;

use chrono::{DateTime, Utc};

use crate::{
    report::{
        entry::WindowEntry,
        period::{Period, PeriodKind},
        scaling::Scaling,
    },
    timeline::model::Segment,
};
use model::{TimelineOverview, TimelinePeriod, TimelineSummary};

/// Analyses the Timeline of `period`. `segments` are every stored Segment overlapping the
/// Period or the Usual's range (in time order); `entries` are the Time Entries overlapping the
/// Period and are shown only for a Day. `now` decides how many Days have passed, and
/// `first_start` (the start of the earliest Segment stored, wherever it lies) how much history
/// the Usual has.
#[must_use]
pub fn summarise(
    period: &Period,
    segments: &[Segment],
    entries: &[WindowEntry],
    now: DateTime<Utc>,
    first_start: Option<DateTime<Utc>>,
) -> TimelineSummary {
    let pieces = clip::clip(segments, period);
    let usual_period = period.usual_range();
    let scaling = Scaling::new(period, now, first_start);
    let figures = figures::measure(&pieces);
    let shape = day_shape::measure(&pieces, period.tz);
    let is_day = period.kind == PeriodKind::Day;
    TimelineSummary {
        period: TimelinePeriod {
            kind: period.kind,
            from: period.from,
            to: period.to,
            usual_from: usual_period.from,
            usual_to: usual_period.to,
            elapsed_days: scaling.elapsed_days,
            usual_available_days: scaling.available_days,
        },
        overview: overview(&figures, &shape),
        usual: usual::compute(&clip::clip(segments, &usual_period), &scaling),
        days: buckets::per_day(&period.days(), &pieces),
        hours: shape.hours.to_vec(),
        apps: apps::rank(&pieces),
        segments: is_day.then(|| pieces.iter().map(clip::Piece::view).collect()),
        entries: is_day.then(|| lane::lane(entries, period.utc_range().into())),
    }
}

fn overview(figures: &figures::Figures, shape: &day_shape::DayShape) -> TimelineOverview {
    let secs = figures.seconds;
    TimelineOverview {
        active_secs: secs.active,
        away_secs: secs.away,
        unknown_secs: secs.unknown,
        switches: figures.switches,
        switches_per_active_hour: switches::per_active_hour(figures.switches, secs.active),
        longest_focus_secs: figures.focus.longest_secs,
        focus_share: figures.focus.share,
        first_active: shape.first_active,
        last_active: shape.last_active,
    }
}
