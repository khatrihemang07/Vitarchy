//! Takes a consistent Snapshot of each tracking app's local database (Toggl Track and
//! Clockify Desktop).
//!
//! This runs as its own signed binary under launchd, so macOS attributes the
//! protected-folder access to it (see docs/adr/0009 and 0010).

pub mod jobs;
pub mod newest;
pub mod paths;
pub mod report;
pub mod run;
pub mod snapshot;
