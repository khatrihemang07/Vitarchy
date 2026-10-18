//! What applying a Read again did, and which Statements an apply touches.

use serde::Serialize;
use utoipa::ToSchema;

/// What applying a Read again did.
#[derive(Debug, Clone, Copy, Default, Serialize, ToSchema)]
pub struct ReadAgainCounts {
    /// Rows added as new Transactions.
    pub added: usize,
    /// Rows this Statement already printed and still finds.
    pub unchanged: usize,
    /// Printings added to Transactions other Statements stored.
    pub printings_added: usize,
    /// Of those, Printings that made the Transaction show more detail.
    pub enriched: usize,
    /// Printings marked no longer found.
    pub no_longer_found: usize,
    /// Payment records linked as a result.
    pub linked: usize,
}

/// Which confirmed Statements an apply touches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Apply {
    /// Always write, even when nothing changed (the per-Statement approval).
    Always,
    /// Write only when rows would be added or marked (Read all again).
    OnlyChanges,
}
