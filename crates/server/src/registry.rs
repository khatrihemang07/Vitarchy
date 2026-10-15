//! The lists of what the server runs. Core features (built in, e.g. settings)
//! and Modules (the user-facing features) both use the `Module` contract.
//!
//! Adding a Module = a new crate under `crates/modules/`, a dependency in this
//! crate's `Cargo.toml`, and one line in `MODULES`.

use vitarchy_core::Module;

const CORE: &[fn() -> Module] = &[vitarchy_settings::feature];

const MODULES: &[fn() -> Module] = &[
    // vitarchy_time::module,
];

/// Every Core feature followed by every Module, in the order they are
/// migrated and mounted.
#[must_use]
pub fn all() -> Vec<Module> {
    CORE.iter().chain(MODULES).map(|build| build()).collect()
}
