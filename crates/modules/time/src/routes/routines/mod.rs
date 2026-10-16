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
    /// What to call the Routine; at most 80 characters, unique ignoring case.
    pub name: String,
    /// 1 to 8 Steps in order.
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
