use std::path::PathBuf;

use super::contract::{CheckOutcome, Direction, Outcome, ReaderResult};

/// The helper's contract fixtures: one set, read by both the helper's tests and these.
fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../helpers/statement-reader/fixtures/contract")
}

fn fixtures() -> Vec<PathBuf> {
    let mut files: Vec<_> = std::fs::read_dir(fixture_dir())
        .expect("contract fixtures directory")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    files
}

#[test]
fn every_contract_fixture_deserialises() {
    let files = fixtures();
    assert!(
        !files.is_empty(),
        "no contract fixtures in {}",
        fixture_dir().display()
    );
    for file in files {
        let text = std::fs::read_to_string(&file).expect("read fixture");
        let parsed: Result<ReaderResult, _> = serde_json::from_str(&text);
        assert!(parsed.is_ok(), "{}: {:?}", file.display(), parsed.err());
    }
}

#[test]
fn a_result_survives_a_round_trip_through_json() {
    for file in fixtures() {
        let text = std::fs::read_to_string(&file).expect("read fixture");
        let first: ReaderResult = serde_json::from_str(&text).expect("parse");
        let again: ReaderResult =
            serde_json::from_str(&serde_json::to_string(&first).expect("serialise"))
                .expect("parse again");
        assert_eq!(first, again, "{}", file.display());
    }
}

#[test]
fn unknown_fields_are_ignored() {
    let json = r#"{"version":1,"outcome":"locked","format":"pdf","future":1,
        "diagnostics":{"stage":"open","code":"x","extra":true}}"#;
    let result: ReaderResult = serde_json::from_str(json).expect("parse");
    assert_eq!(result.outcome, Outcome::Locked);
    assert!(result.statements.is_empty());
}

#[test]
fn enums_use_the_wire_spelling() {
    assert_eq!(Outcome::NoTable.id(), "no_table");
    assert_eq!(Direction::Credit.id(), "credit");
    assert_eq!(CheckOutcome::Skipped.id(), "skipped");
    assert_eq!(
        serde_json::to_string(&Outcome::NoTable).unwrap(),
        "\"no_table\""
    );
    assert_eq!(Outcome::from_id("no_table"), Some(Outcome::NoTable));
    assert_eq!(Outcome::from_id("nope"), None);
}

#[test]
fn a_payment_record_carries_its_instrument_issuer_and_digits_when_given() {
    let json = r#"{"occurred_at":"2026-10-18T11:20:00","description":"d","amount_paise":5,
        "direction":"debit","instrument":"ICICI R","instrument_issuer":"ICICI Bank",
        "instrument_digits":"8765"}"#;
    let record: super::contract::PaymentRecord = serde_json::from_str(json).expect("parse");
    assert_eq!(record.instrument_issuer.as_deref(), Some("ICICI Bank"));
    assert_eq!(record.instrument_digits.as_deref(), Some("8765"));
}

#[test]
fn a_payment_record_from_an_older_helper_has_no_instrument_details() {
    let json = r#"{"occurred_at":"2026-10-18T11:20:00","description":"d","amount_paise":5,
        "direction":"debit","instrument":"XXXXXX4321"}"#;
    let record: super::contract::PaymentRecord = serde_json::from_str(json).expect("parse");
    assert_eq!(
        (record.instrument_issuer, record.instrument_digits),
        (None, None)
    );
}

#[test]
fn the_api_schema_spells_enums_as_the_wire_does() {
    use utoipa::PartialSchema;

    use super::contract::AccountKind;

    let spelled = |schema: utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>| {
        serde_json::to_value(schema).unwrap()["enum"].clone()
    };
    assert_eq!(
        spelled(Direction::schema()),
        serde_json::json!([Direction::Debit.id(), Direction::Credit.id()])
    );
    assert_eq!(
        spelled(AccountKind::schema())[1],
        serde_json::json!(AccountKind::CreditCard.id())
    );
}
