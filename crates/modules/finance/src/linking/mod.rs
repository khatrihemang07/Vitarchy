//! Linking Transactions into Transfers in the database (ADR 0017): running detection when
//! something changed, the one-time backfill, links made by hand, and hiding Accounts. The
//! rules themselves are pure and live in [`crate::transfers`].

mod after;
mod backfill;
mod candidates;
mod manual;
mod run;
mod trigger;
mod visibility;
mod window;

pub use after::{after_accounts, after_everything, after_statement};
pub use backfill::{BACKFILL_MARKER, backfill_on_start, backfill_once};
pub use candidates::candidates_for;
pub use manual::link;
pub use trigger::{Cause, Trigger};
pub use visibility::set_hidden;
