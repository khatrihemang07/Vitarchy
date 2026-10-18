use std::path::Path;

use vitarchy_finance::reader::{ReaderError, ReaderResult, StatementReader, testing::FakeReader};

fn result(outcome: &str) -> ReaderResult {
    serde_json::from_str(&format!(
        r#"{{"version":2,"outcome":"{outcome}","format":null,"diagnostics":{{}}}}"#
    ))
    .unwrap()
}

#[tokio::test]
async fn answers_in_order_then_repeats_the_last_and_records_calls() {
    let fake = FakeReader::with_results([result("locked"), result("read")]);
    let passwords = vec!["a".to_owned()];
    for expected in ["locked", "read", "read"] {
        let answer = fake
            .read(Path::new("/f"), &passwords, Path::new("/o"), Some("r-1"))
            .await
            .unwrap();
        assert_eq!(answer.outcome.id(), expected);
    }
    assert_eq!(fake.calls().len(), 3);
    assert_eq!(fake.calls()[0].password_count, 1);
    assert_eq!(fake.calls()[0].request_id.as_deref(), Some("r-1"));
}

#[tokio::test]
async fn can_be_told_to_fail() {
    let fake = FakeReader::with_answers([Err(ReaderError::Timeout)]);
    let error = fake
        .read(Path::new("/f"), &[], Path::new("/o"), None)
        .await
        .unwrap_err();
    assert!(matches!(error, ReaderError::Timeout));
}
