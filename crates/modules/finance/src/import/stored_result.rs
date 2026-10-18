//! A reader answer in the shape the `statements` table keeps it.

use serde_json::Value;

use crate::{
    reader::{ReaderError, ReaderResult},
    repo::statements::NewStatement,
};

/// The columns that come from a read: outcome, format, kind, the result and the version.
pub struct StoredResult {
    outcome: String,
    format: Option<String>,
    kind: Option<String>,
    result: Option<Value>,
    version: Option<String>,
}

impl StoredResult {
    /// What to store for this answer; a failed run is kept as `crashed`.
    #[must_use]
    pub fn of(answer: &Result<ReaderResult, ReaderError>) -> Self {
        match answer {
            Ok(result) => Self::from_result(result),
            Err(_) => Self {
                outcome: "crashed".to_owned(),
                format: None,
                kind: None,
                result: None,
                version: None,
            },
        }
    }

    /// What to store for a result the reader returned.
    #[must_use]
    pub fn from_result(result: &ReaderResult) -> Self {
        Self {
            outcome: result.outcome.id().to_owned(),
            format: result.format.map(|f| f.id().to_owned()),
            kind: result.kind.map(|k| k.id().to_owned()),
            result: serde_json::to_value(result).ok(),
            version: Some(result.version.to_string()),
        }
    }

    /// The row to insert or update with, for a file named `file_name` stored as `sha`.
    #[must_use]
    pub fn row<'a>(&'a self, sha: &'a str, file_name: &'a str) -> NewStatement<'a> {
        NewStatement {
            file_sha256: sha,
            file_name,
            format: self.format.as_deref(),
            kind: self.kind.as_deref(),
            outcome: &self.outcome,
            result: self.result.as_ref(),
            reader_version: self.version.as_deref(),
        }
    }
}
