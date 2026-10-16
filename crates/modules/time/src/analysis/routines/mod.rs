//! Routines: ordered Steps the person goes through in a Day, found in their Time Entries.
//!
//! Everything here is pure. A Routine lives inside one Day, so an entry belongs to the Day it
//! **starts** in (never its per-Day portions): a night's sleep from 23:00 is one entry of the
//! Day it began, and its second half never starts a sequence on the next Day.

mod draft;
mod occurrence;
mod routine;
mod stats;
mod step;
mod suggest;

pub use draft::{RoutineDraft, RoutineError};
pub use occurrence::{Occurrence, StepSpan, occurrences};
pub use routine::Routine;
pub use stats::{RoutineStats, StepStat, WeekdayStat, routine_stats};
pub use step::{MatchKind, Step};
pub use suggest::{MAX_STEPS, MAX_SUGGESTIONS, Suggestion, suggest};

/// Test helper: an entry in UTC with a project and description.
#[cfg(test)]
pub(super) fn entry(
    project: &str,
    description: &str,
    start: &str,
    end: &str,
) -> super::lifelog::LifeEntry {
    super::lifelog::LifeEntry::new(
        (project != "-").then_some(project),
        None,
        description,
        start.parse().unwrap(),
        end.parse().unwrap(),
    )
}
