//! A Payment record: what a payment app noted about one payment.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::enums::Direction;

/// What a payment app noted about one payment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentRecord {
    /// Local time, no offset.
    pub occurred_at: NaiveDateTime,
    /// With whom.
    pub description: String,
    /// Always greater than 0.
    pub amount_paise: i64,
    /// Which way the money moved.
    pub direction: Direction,
    /// The app's own id.
    #[serde(default)]
    pub source_id: Option<String>,
    /// The bank reference.
    #[serde(default)]
    pub utr: Option<String>,
    /// The funding account's mask or label as printed.
    #[serde(default)]
    pub instrument: Option<String>,
    /// The issuer the helper recognised in `instrument`, e.g. `ICICI Bank`.
    #[serde(default)]
    pub instrument_issuer: Option<String>,
    /// The trailing visible digits of `instrument`.
    #[serde(default)]
    pub instrument_digits: Option<String>,
}
