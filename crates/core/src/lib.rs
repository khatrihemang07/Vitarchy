//! Shared ground for the Vitarchy server: the Module contract and cross-cutting
//! infrastructure. This crate holds no domain types.

mod api_json;
mod error;
#[cfg(any(test, feature = "test-support"))]
pub mod log_capture;
mod log_level;
mod module;
mod request_id;
mod state;
pub mod telemetry;
#[cfg(any(test, feature = "test-support"))]
pub mod test_db;

pub use api_json::ApiJson;
pub use error::{ApiError, Problem};
pub use log_level::level_for_status;
pub use module::Module;
pub use request_id::{REQUEST_ID_HEADER, RequestId};
pub use state::AppState;
