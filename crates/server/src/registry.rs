//! The single list of Modules. Adding a Module = a new crate under
//! `crates/modules/`, a dependency in this crate's `Cargo.toml`, and one line below.

use vitarchy_core::Module;

const MODULES: &[fn() -> Module] = &[
    // vitarchy_time::module,
];

#[must_use]
pub fn modules() -> Vec<Module> {
    MODULES.iter().map(|build| build()).collect()
}
