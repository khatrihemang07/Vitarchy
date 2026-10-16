//! A Routine as the person submits it, checked before it is stored.

use super::Step;

/// The most Steps a Routine may have.
pub const MAX_STEPS_PER_ROUTINE: usize = 8;
/// The longest Routine name, in characters.
pub const MAX_NAME_CHARS: usize = 80;
/// The longest Step label, in characters.
pub const MAX_LABEL_CHARS: usize = 80;
/// The longest Step match value, in characters.
pub const MAX_VALUE_CHARS: usize = 200;

/// Why a Routine could not be saved. Messages hold no entered values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum RoutineError {
    /// The name is empty.
    #[error("a Routine needs a name")]
    NameEmpty,
    /// The name is longer than [`MAX_NAME_CHARS`].
    #[error("a Routine name can be at most {MAX_NAME_CHARS} characters")]
    NameTooLong,
    /// There are no Steps.
    #[error("a Routine needs at least one Step")]
    NoSteps,
    /// More than [`MAX_STEPS_PER_ROUTINE`] Steps.
    #[error("a Routine can have at most {MAX_STEPS_PER_ROUTINE} Steps")]
    TooManySteps,
    /// A Step has no label.
    #[error("every Step needs a label")]
    LabelEmpty,
    /// A Step has no description or project to match.
    #[error("every Step needs a description or project to match")]
    ValueEmpty,
    /// A label or value is too long.
    #[error(
        "a Step label is at most {MAX_LABEL_CHARS} characters and a match at most {MAX_VALUE_CHARS}"
    )]
    StepTooLong,
    /// Another Routine has this name.
    #[error("another Routine already has this name")]
    NameTaken,
    /// No Routine has the id.
    #[error("there is no such Routine")]
    NotFound,
}

impl RoutineError {
    /// Stable machine label for the problem's `code`.
    #[must_use]
    pub fn code(self) -> &'static str {
        match self {
            Self::NameEmpty => "routine_name_empty",
            Self::NameTooLong => "routine_name_too_long",
            Self::NoSteps => "routine_no_steps",
            Self::TooManySteps => "routine_too_many_steps",
            Self::LabelEmpty => "routine_label_empty",
            Self::ValueEmpty => "routine_value_empty",
            Self::StepTooLong => "routine_step_too_long",
            Self::NameTaken => "routine_name_taken",
            Self::NotFound => "routine_not_found",
        }
    }
}

/// A Routine's name and Steps, trimmed and valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutineDraft {
    /// The name, trimmed.
    pub name: String,
    /// 1 to [`MAX_STEPS_PER_ROUTINE`] Steps, texts trimmed.
    pub steps: Vec<Step>,
}

impl RoutineDraft {
    /// Trims and checks a submitted name and Steps.
    ///
    /// # Errors
    /// The first [`RoutineError`] found: name, then Step count, then each Step in order.
    pub fn new(name: &str, steps: Vec<Step>) -> Result<Self, RoutineError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(RoutineError::NameEmpty);
        }
        if name.chars().count() > MAX_NAME_CHARS {
            return Err(RoutineError::NameTooLong);
        }
        if steps.is_empty() {
            return Err(RoutineError::NoSteps);
        }
        if steps.len() > MAX_STEPS_PER_ROUTINE {
            return Err(RoutineError::TooManySteps);
        }
        let steps = steps
            .into_iter()
            .map(|step| {
                let (label, value) = (step.label.trim(), step.value.trim());
                if label.is_empty() {
                    return Err(RoutineError::LabelEmpty);
                }
                if value.is_empty() {
                    return Err(RoutineError::ValueEmpty);
                }
                if label.chars().count() > MAX_LABEL_CHARS
                    || value.chars().count() > MAX_VALUE_CHARS
                {
                    return Err(RoutineError::StepTooLong);
                }
                Ok(Step {
                    label: label.to_owned(),
                    kind: step.kind,
                    value: value.to_owned(),
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            name: name.to_owned(),
            steps,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::MatchKind;
    use super::*;

    fn step(label: &str, value: &str) -> Step {
        Step {
            label: label.into(),
            kind: MatchKind::Description,
            value: value.into(),
        }
    }

    fn error(name: &str, steps: Vec<Step>) -> RoutineError {
        RoutineDraft::new(name, steps).unwrap_err()
    }

    #[test]
    fn a_valid_routine_is_trimmed() {
        let draft = RoutineDraft::new(" Office day ", vec![step(" Trip ", " travel ")]).unwrap();
        assert_eq!(draft.name, "Office day");
        assert_eq!(
            (draft.steps[0].label.as_str(), draft.steps[0].value.as_str()),
            ("Trip", "travel")
        );
    }

    #[test]
    fn each_rule_has_its_own_error() {
        let one = || vec![step("a", "b")];
        assert_eq!(error("  ", one()), RoutineError::NameEmpty);
        assert_eq!(error(&"x".repeat(81), one()), RoutineError::NameTooLong);
        assert_eq!(error("n", vec![]), RoutineError::NoSteps);
        assert_eq!(
            error("n", vec![step("a", "b"); 9]),
            RoutineError::TooManySteps
        );
        assert_eq!(error("n", vec![step(" ", "b")]), RoutineError::LabelEmpty);
        assert_eq!(error("n", vec![step("a", " ")]), RoutineError::ValueEmpty);
        assert_eq!(
            error("n", vec![step(&"l".repeat(81), "b")]),
            RoutineError::StepTooLong
        );
    }

    #[test]
    fn eight_steps_and_an_eighty_character_name_are_fine() {
        assert!(RoutineDraft::new(&"x".repeat(80), vec![step("a", "b"); 8]).is_ok());
    }

    #[test]
    fn every_error_has_a_distinct_code() {
        let all = [
            RoutineError::NameEmpty,
            RoutineError::NameTooLong,
            RoutineError::NoSteps,
            RoutineError::TooManySteps,
            RoutineError::LabelEmpty,
            RoutineError::ValueEmpty,
            RoutineError::StepTooLong,
            RoutineError::NameTaken,
            RoutineError::NotFound,
        ];
        let codes: std::collections::HashSet<_> = all.iter().map(|e| e.code()).collect();
        assert_eq!(codes.len(), all.len());
    }
}
