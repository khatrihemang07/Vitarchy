//! Everything about talking to Toggl Track: the API seam, its HTTP
//! implementation, the payload shapes and how failures are classified.

pub mod api;
pub mod client;
pub mod error;
pub mod types;
