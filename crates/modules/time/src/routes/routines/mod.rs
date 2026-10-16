//! `/api/time/routines...`: suggested and saved Routines, and how they went.

pub mod delete;
pub mod list;
pub mod occurrences;
pub mod save;
pub mod suggestions;

use serde::Deserialize;
use utoipa::ToSchema;

use crate::analysis::routines::{RoutineDraft, RoutineError, Step};

/// Body of `POST /routines` and `PUT /routines/{id}`.
#[derive(Deserialize, ToSchema)]
pub struct RoutineRequest {
    /// What to call the Routine; unique ignoring case (length limits: see `RoutineDraft`).
    pub name: String,
    /// The Steps in order; at least one.
    pub steps: Vec<Step>,
}

impl RoutineRequest {
    /// The checked Routine this asks for.
    ///
    /// # Errors
    /// The [`RoutineError`] for the first thing wrong with it.
    pub fn draft(self) -> Result<RoutineDraft, RoutineError> {
        RoutineDraft::new(&self.name, self.steps)
    }
}
