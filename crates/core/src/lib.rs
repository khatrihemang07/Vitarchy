//! Shared ground for the Vitarchy server: the Module contract and cross-cutting
//! infrastructure. This crate holds no domain types.

mod api_json;
mod error;
mod module;
mod state;
pub mod telemetry;

pub use api_json::ApiJson;
pub use error::{ApiError, Problem};
pub use module::Module;
pub use state::AppState;
