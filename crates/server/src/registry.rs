//! The lists of what the server runs. Core features (built in, e.g. settings)
//! and Modules (the areas of life) both use the `Module` contract.
//!
//! Adding a Module = a new crate under `crates/modules/` (listed in the root
//! `Cargo.toml` members), a dependency in this crate's `Cargo.toml`, and one
//! line in `MODULES`.

use vitarchy_core::Module;

const CORE: &[fn() -> Module] = &[vitarchy_settings::feature];

const MODULES: &[fn() -> Module] = &[
    // vitarchy_time::module,
];

/// The Core features (built in), in the order they are migrated and mounted.
#[must_use]
pub fn core() -> Vec<Module> {
    CORE.iter().map(|build| build()).collect()
}

/// The Modules, in the order they are migrated and mounted.
#[must_use]
pub fn modules() -> Vec<Module> {
    MODULES.iter().map(|build| build()).collect()
}

/// Every Core feature followed by every Module.
#[must_use]
pub fn all() -> Vec<Module> {
    core().into_iter().chain(modules()).collect()
}
