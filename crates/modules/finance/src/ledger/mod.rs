//! The Transactions page's filters and figures, as pure logic: reading the query, the LIKE
//! pattern for the description search, the Previous Period's range, and the Overview sums.
//! The SQL that feeds them lives in `repo`.

mod like;
mod query;
mod range;
mod sums;

pub use query::{Filter, Flow, LedgerQuery, Sort};
pub use range::{MAX_DAYS, days, previous_range};
pub use sums::{Membership, Row, Totals, aggregate};

#[cfg(test)]
mod query_tests;
#[cfg(test)]
mod sums_tests;
