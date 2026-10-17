//! Turns stored rows into the [`Inputs`] the Insight kinds read: one measured series per
//! Routine Step, Routine total, frequent description, top app and top Context.
//!
//! Everything is measured over a *reach* that extends the Period back far enough to fill the
//! earlier change window, so Insights about a short Period (even one Day) can still compare the
//! last four weeks with the four before. Habits (which descriptions, apps and Contexts are
//! worth a series) are picked over the whole reach for descriptions and over the report for
//! screen time; typical times, weekday differences and outliers are then read from the
//! report's own Days only, while changes are read from the windows.
//! Only finished Days are measured (see [`windows`]).

mod descriptions;
mod measure;
mod routines;
mod screen;
mod windows;

use std::collections::HashSet;

use chrono::{DateTime, Utc};

use super::inputs::Inputs;
use crate::analysis::{
    Period,
    activity::ActivityRow,
    lifelog::{LifeEntry, fold},
    routines::{MatchKind, Routine},
};

/// The stored rows Insights are found in.
pub struct Sources<'a> {
    /// Time Entries touching the reach.
    pub entries: &'a [LifeEntry],
    /// Activity touching the reach.
    pub activity: &'a [ActivityRow],
    /// Every saved Routine.
    pub routines: &'a [Routine],
    /// The present moment: a Day that has not finished is left out of every series.
    pub now: DateTime<Utc>,
}

pub use windows::{reach_period, windows};

/// Everything measured in `sources`, with the report and change windows.
#[must_use]
pub fn gather(report: &Period, reach: &Period, sources: &Sources) -> Inputs {
    let all = windows(report, sources.now);
    let covered: HashSet<String> = sources
        .routines
        .iter()
        .flat_map(|r| &r.steps)
        .filter(|step| step.kind == MatchKind::Description)
        .map(|step| fold(&step.value))
        .collect();
    let mut series = routines::series(reach, sources.routines, sources.entries);
    series.extend(descriptions::series(reach, sources.entries, &covered));
    series.extend(screen::series(
        report,
        reach,
        sources.activity,
        all.recent.to,
    ));
    Inputs {
        report: all.report,
        recent: all.recent,
        earlier: all.earlier,
        series,
    }
}
