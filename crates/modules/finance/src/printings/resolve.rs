//! The view of a Transaction, derived from its Printings (ADR 0015). Pure: the same
//! Printings always give the same view, so the rule can change and be re-applied.

use super::Printing;
use crate::dedupe::normalise;

/// The fields of a Transaction that its Printings fill; amount, date and direction are
/// fixed when the Transaction is created and are not here.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Detail {
    /// The fullest description.
    pub description: String,
    /// Balance after.
    pub balance_paise: Option<i64>,
    /// A UPI or NEFT reference.
    pub reference: Option<String>,
    /// An id the issuer printed.
    pub source_id: Option<String>,
    /// Foreign amount as printed.
    pub foreign_amount: Option<String>,
    /// Foreign currency code.
    pub foreign_currency: Option<String>,
}

/// What a Transaction shows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resolved {
    /// The merged fields.
    pub detail: Detail,
    /// True only when every Printing is gone.
    pub no_longer_found: bool,
}

/// Merges `printings` (in creation order). Gone ones are ignored while any live one exists.
/// The description starts as the first one's and is replaced only by a strictly extending
/// one; every other field is the first value present. An empty list resolves to nothing.
#[must_use]
pub fn resolve(printings: &[Printing]) -> Resolved {
    let live: Vec<&Printing> = printings.iter().filter(|p| !p.gone).collect();
    let counted: Vec<&Printing> = if live.is_empty() {
        printings.iter().collect()
    } else {
        live
    };
    let first = |pick: fn(&Printing) -> Option<String>| counted.iter().find_map(|p| pick(p));
    Resolved {
        detail: Detail {
            description: fullest_description(&counted),
            balance_paise: counted.iter().find_map(|p| p.printed.balance_paise),
            reference: first(|p| p.printed.reference.clone()),
            source_id: first(|p| p.printed.source_id.clone()),
            foreign_amount: first(|p| p.printed.foreign_amount.clone()),
            foreign_currency: first(|p| p.printed.foreign_currency.clone()),
        },
        no_longer_found: !printings.is_empty() && printings.iter().all(|p| p.gone),
    }
}

fn fullest_description(printings: &[&Printing]) -> String {
    let mut shown = printings
        .first()
        .map_or("", |p| p.printed.description.as_str());
    let mut shown_key = normalise(shown);
    for next in &printings[printings.len().min(1)..] {
        let key = normalise(&next.printed.description);
        if key.len() > shown_key.len() && key.contains(&shown_key) {
            shown = &next.printed.description;
            shown_key = key;
        }
    }
    shown.to_owned()
}
