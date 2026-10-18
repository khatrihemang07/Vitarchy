//! The helper's whole answer for one file.

use serde::{Deserialize, Serialize};

use super::{
    enums::{Format, Kind, Outcome},
    payment::PaymentRecord,
    statement::Statement,
};

/// The helper's whole answer for one file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReaderResult {
    /// Contract version.
    pub version: u32,
    /// What became of the file.
    pub outcome: Outcome,
    /// The file's format; `None` when unsupported.
    pub format: Option<Format>,
    /// Whether the file was encrypted.
    #[serde(default)]
    pub encrypted: bool,
    /// Index into the passwords tried of the one that worked.
    #[serde(default)]
    pub unlocked_with: Option<usize>,
    /// Set only when `outcome` is `read`.
    #[serde(default)]
    pub kind: Option<Kind>,
    /// Filled when `kind` is `statement`.
    #[serde(default)]
    pub statements: Vec<Statement>,
    /// Filled when `kind` is `payment_history`.
    #[serde(default)]
    pub payment_records: Vec<PaymentRecord>,
    /// Where it stopped and counts; holds no content.
    pub diagnostics: Diagnostics,
}

/// Stage, reason and counts for debugging.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnostics {
    /// When the outcome is not `read`: `open`, `read`, `locate`, `segment` or `normalize`.
    #[serde(default)]
    pub stage: Option<String>,
    /// A stable `snake_case` reason.
    #[serde(default)]
    pub code: Option<String>,
    /// Pages read.
    #[serde(default)]
    pub pages: Option<u32>,
    /// Rows found.
    #[serde(default)]
    pub rows: Option<u32>,
}
