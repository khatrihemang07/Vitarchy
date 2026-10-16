//! Sync: bringing Time Entries from Toggl into Vitarchy so the copy matches Toggl.

pub(crate) mod account;
mod apply;
mod counts;
mod fetch;
pub(crate) mod plan;
mod run;
mod window;

pub use counts::SyncCounts;
pub use run::run_sync;
