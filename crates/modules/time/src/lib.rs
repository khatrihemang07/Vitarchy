//! Time: the Module that mirrors the person's Toggl Track Time Entries and Imports Activity
//! from several Sources (Toggl, Clockify; ADR 0010). Owns the `time` Postgres schema.
//! Only the server talks to Toggl (ADR 0008).

mod activity;
mod error;
mod repo;
mod report;
mod routes;
mod saturating;
mod sync;
pub mod toggl;

use vitarchy_core::Module;

pub use activity::{SnapshotFile, Snapshots, Source};
pub use error::{ProblemBody, TimeError};
pub use routes::router_with;

/// The Time Module, mounted at `/api/time`.
#[must_use]
pub fn module() -> Module {
    Module::new("time", routes::router, sqlx::migrate!("./migrations"))
}
