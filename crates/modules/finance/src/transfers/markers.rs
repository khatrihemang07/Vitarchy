//! The payee and receipt markers of card-bill payments, kept as data in `markers.toml`.

use std::sync::OnceLock;

use serde::Deserialize;

use super::tokens::spaced;

const BUILT_IN: &str = include_str!("markers.toml");

/// What marks a card-bill payment and its receipts, ready to match against descriptions.
#[derive(Debug, Clone, Deserialize)]
pub struct Markers {
    payee: Vec<String>,
    receipt: Vec<String>,
}

impl Markers {
    /// The markers shipped in `markers.toml`.
    ///
    /// # Panics
    /// Never in practice: the file is embedded at compile time, so what is parsed is what
    /// shipped, and `the_shipped_file_parses_and_knows_both_kinds` parses it on every test run.
    #[must_use]
    pub fn built_in() -> &'static Self {
        static MARKERS: OnceLock<Markers> = OnceLock::new();
        // Cannot fail: BUILT_IN is `include_str!`, fixed at compile time and parsed by a test.
        MARKERS.get_or_init(|| Self::parse(BUILT_IN).expect("markers.toml parses"))
    }

    /// Reads markers from TOML text with `payee` and `receipt` lists.
    ///
    /// # Errors
    /// The text is not valid TOML of that shape.
    pub fn parse(text: &str) -> Result<Self, toml::de::Error> {
        let raw: Self = toml::from_str(text)?;
        Ok(Self {
            payee: raw.payee.iter().map(|m| spaced(m)).collect(),
            receipt: raw.receipt.iter().map(|m| spaced(m)).collect(),
        })
    }

    /// Whether any description names a card-bill payee.
    #[must_use]
    pub fn is_payee(&self, texts: &[String]) -> bool {
        any_marker(&self.payee, texts)
    }

    /// Whether any description reads as a card-bill payment received.
    #[must_use]
    pub fn is_receipt(&self, texts: &[String]) -> bool {
        any_marker(&self.receipt, texts)
    }
}

fn any_marker(markers: &[String], texts: &[String]) -> bool {
    texts.iter().any(|text| {
        let text = spaced(text);
        markers.iter().any(|marker| text.contains(marker.as_str()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(text: &str) -> Vec<String> {
        vec![text.to_owned()]
    }

    #[test]
    fn the_shipped_file_parses_and_knows_both_kinds() {
        let markers = Markers::built_in();
        assert!(markers.is_payee(&one("UPI/CRED Club/bill")));
        assert!(markers.is_payee(&one("pay@cred.club")));
        assert!(markers.is_receipt(&one("BBPS Payment received")));
        assert!(markers.is_receipt(&one("BBPS-PAYMENT")));
        assert!(markers.is_receipt(&one("PAYMENT RECEIVED 12345")));
    }

    #[test]
    fn ordinary_descriptions_are_neither() {
        let markers = Markers::built_in();
        assert!(!markers.is_payee(&one("UPI/Grocer/lunch")));
        assert!(!markers.is_receipt(&one("POS purchase")));
    }

    #[test]
    fn any_description_of_a_transaction_can_carry_the_marker() {
        let texts = vec!["short".to_owned(), "short and BBPS detail".to_owned()];
        assert!(Markers::built_in().is_receipt(&texts));
    }

    #[test]
    fn custom_markers_parse_and_bad_text_is_refused() {
        let markers = Markers::parse("payee = [\"acme pay\"]\nreceipt = [\"got it\"]").unwrap();
        assert!(markers.is_payee(&one("ACME-PAY 1")));
        assert!(markers.is_receipt(&one("Got It!")));
        assert!(Markers::parse("payee = 3").is_err());
    }
}
