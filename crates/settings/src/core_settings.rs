use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use utoipa::ToSchema;
use vitarchy_core::ApiError;

const DISPLAY_NAME_MAX_CHARS: usize = 50;

#[derive(Clone, Copy, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Motion {
    #[default]
    System,
    Reduced,
    Full,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WeekStart {
    #[default]
    Monday,
    Sunday,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Clock {
    #[default]
    System,
    H12,
    H24,
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DateFormat {
    #[default]
    System,
    DayMonthYear,
    MonthDayYear,
    Iso,
}

/// Settings that apply across all of Vitarchy. Missing keys fall back to their
/// defaults, so adding a setting later needs no data migration.
#[derive(Clone, Default, Serialize, Deserialize, ToSchema)]
#[serde(default)]
pub struct CoreSettings {
    /// What Vitarchy calls the user; trimmed, at most 50 characters.
    pub display_name: String,
    pub theme: Theme,
    pub motion: Motion,
    pub week_start: WeekStart,
    pub clock: Clock,
    pub date_format: DateFormat,
}

/// One field of a stored object, or its default if missing or no longer valid.
fn field_or_default<T: DeserializeOwned + Default>(stored: &Value, key: &str) -> T {
    stored
        .get(key)
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

impl CoreSettings {
    /// Reads a stored document leniently, field by field: a missing or
    /// invalid value resets only that field. (PUT stays strict.)
    #[must_use]
    pub fn from_stored(stored: &Value) -> Self {
        Self {
            display_name: field_or_default(stored, "display_name"),
            theme: field_or_default(stored, "theme"),
            motion: field_or_default(stored, "motion"),
            week_start: field_or_default(stored, "week_start"),
            clock: field_or_default(stored, "clock"),
            date_format: field_or_default(stored, "date_format"),
        }
    }

    /// Normalises the settings (trims the display name) and rejects invalid values.
    ///
    /// # Errors
    /// [`ApiError::BadRequest`] when the display name exceeds 50 characters.
    pub fn validated(mut self) -> Result<Self, ApiError> {
        self.display_name = self.display_name.trim().to_owned();
        if self.display_name.chars().count() > DISPLAY_NAME_MAX_CHARS {
            return Err(ApiError::BadRequest(format!(
                "display_name must be at most {DISPLAY_NAME_MAX_CHARS} characters"
            )));
        }
        Ok(self)
    }
}
