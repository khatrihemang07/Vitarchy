//! Activity Import: bringing Activity from the latest Snapshot into Vitarchy.
//! Append-only, so Vitarchy keeps Activity that Toggl later discards.

mod counts;
mod error;
mod import;
pub(crate) mod parse;
mod read;
mod snapshot_path;

pub use counts::ImportCounts;
pub use error::ActivityError;
pub use import::run_import;
pub use snapshot_path::SnapshotFile;
