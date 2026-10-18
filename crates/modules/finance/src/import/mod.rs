//! The import flow: reading an upload into an unconfirmed Statement, confirming it, and
//! reading a stored Statement again. Handlers call these; they hold the flow, not the HTTP.

pub mod choice;
pub mod confirm;
pub mod intake;
mod link_waiting;
mod proposal;
pub mod read_again;
pub mod read_again_all;
pub mod read_again_diff;
mod read_again_preview;
mod read_counts;
pub mod reading;
mod resolve;
pub mod retry;
mod stored_result;
