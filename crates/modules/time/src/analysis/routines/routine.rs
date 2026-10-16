//! A saved Routine.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use super::Step;

/// A Routine the person confirmed or created.
#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Eq)]
pub struct Routine {
    /// Identifier, stable across edits.
    pub id: i64,
    /// What the person calls it.
    pub name: String,
    /// The Steps in order.
    pub steps: Vec<Step>,
    /// When it was saved.
    pub created_at: DateTime<Utc>,
    /// When it was last changed.
    pub updated_at: DateTime<Utc>,
}
