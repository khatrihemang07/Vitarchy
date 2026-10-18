//! What is known about one Transaction row when deciding whether two rows are the same.
//! Incoming and stored rows both become [`Evidence`], so one rule serves both.

use chrono::NaiveDate;

use super::origin::Origins;
use crate::reader::contract::{Direction, Transaction};

/// The facts of a row that can show it is (or is not) another row, already normalised.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    /// The printed date.
    pub date: NaiveDate,
    /// The amount, above 0.
    pub amount_paise: i64,
    /// Which way the money moved.
    pub direction: Direction,
    /// An id the issuer printed, trimmed; `None` when absent or blank.
    pub source_id: Option<String>,
    /// Balance after, when printed.
    pub balance_paise: Option<i64>,
    /// A UPI or NEFT reference, trimmed; `None` when absent or blank.
    pub reference: Option<String>,
    /// The description with only alphanumerics kept, case folded.
    pub description: String,
    /// The kinds of file that printed it; [`Origins::Unknown`] when not known.
    pub origins: Origins,
}

impl Evidence {
    /// Normalises the raw facts of a row.
    #[must_use]
    pub fn new(
        date: NaiveDate,
        amount_paise: i64,
        direction: Direction,
        source_id: Option<&str>,
        balance_paise: Option<i64>,
        reference: Option<&str>,
        description: &str,
    ) -> Self {
        Self {
            date,
            amount_paise,
            direction,
            source_id: present(source_id),
            balance_paise,
            reference: present(reference),
            description: normalise(description),
            origins: Origins::Unknown,
        }
    }

    /// The same facts, printed by files of these `origins`.
    #[must_use]
    pub fn with_origins(mut self, origins: Origins) -> Self {
        self.origins = origins;
        self
    }
}

impl From<&Transaction> for Evidence {
    fn from(tx: &Transaction) -> Self {
        Self::new(
            tx.date,
            tx.amount_paise,
            tx.direction,
            tx.source_id.as_deref(),
            tx.balance_paise,
            tx.reference.as_deref(),
            &tx.description,
        )
    }
}

fn present(text: Option<&str>) -> Option<String> {
    text.map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

/// Text with only alphanumerics kept, case folded: how descriptions are compared.
#[must_use]
pub fn normalise(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
