//! Settings: a Core feature built on the Module contract. Stores one JSON
//! document per scope (`core`, or a Module id) in the `settings` schema.

mod changes;
mod core_settings;
mod repo;
mod routes;
mod save;

use vitarchy_core::Module;

pub use core_settings::{Clock, CoreSettings, DateFormat, Motion, Theme, WeekStart};

/// The settings feature, mounted at `/api/settings`.
#[must_use]
pub fn feature() -> Module {
    Module::new("settings", routes::router, sqlx::migrate!("./migrations"))
}
