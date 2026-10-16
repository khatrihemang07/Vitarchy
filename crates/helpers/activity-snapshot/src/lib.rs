//! Takes a consistent Snapshot of the Toggl Track app's local database.
//!
//! This runs as its own signed binary under launchd, so macOS attributes the
//! protected-folder access to it (see docs/adr/0009).

pub mod paths;
pub mod report;
pub mod snapshot;
