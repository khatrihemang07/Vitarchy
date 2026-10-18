//! Activity Import: bringing Activity from each Source's latest Snapshot into Vitarchy.
//! Append-only, so Vitarchy keeps Activity that a Source later discards. Each row is kept
//! exactly as its Source recorded it; combining Sources is left to whatever reads them.

mod clockify;
mod core_data;
mod counts;
mod dedupe;
mod error;
mod import;
mod model;
mod read_snapshot;
mod snapshot_path;
mod snapshots;
mod source;
mod strengths;
mod toggl;

pub use counts::ImportCounts;
pub use error::ActivityError;
pub use import::run_import;
pub use model::{Activity, Extras};
pub use snapshot_path::SnapshotFile;
pub use snapshots::Snapshots;
pub use source::Source;
pub use strengths::Strengths;
