//! Serde mirror of the statement-reader contract, version 1
//! (`crates/helpers/statement-reader/CONTRACT.md`). Unknown fields are ignored so a newer
//! helper can add fields without breaking an older server.

mod enums;
mod payment;
mod result;
mod statement;
mod wire_enum;

pub use enums::{AccountKind, CheckOutcome, Direction, Format, Kind, Outcome};
pub use payment::PaymentRecord;
pub use result::{Diagnostics, ReaderResult};
pub use statement::{Check, Period, ProposedAccount, Statement, Summary, Transaction};

/// The contract version this server speaks.
pub const VERSION: u32 = 1;
