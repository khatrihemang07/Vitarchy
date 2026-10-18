//! `ProcessReader` against tiny shell scripts standing in for the helper.
#![cfg(unix)]

use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Duration,
};

use vitarchy_finance::reader::{ProcessReader, ReaderError, StatementReader};

const LOCKED: &str = r#"{"version":2,"outcome":"locked","format":"pdf","encrypted":true,"diagnostics":{"stage":"open","code":"no_password_worked"}}"#;

fn script(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("fake-reader");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

async fn read(
    reader: &ProcessReader,
    passwords: &[String],
) -> Result<vitarchy_finance::reader::ReaderResult, ReaderError> {
    reader
        .read(
            Path::new("/in/file.pdf"),
            passwords,
            Path::new("/out/copy"),
            Some("req-42"),
        )
        .await
}

#[tokio::test]
async fn sends_the_request_on_stdin_and_parses_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let seen = dir.path().join("stdin.json");
    let program = script(
        dir.path(),
        &format!("cat > '{}'\necho '{LOCKED}'", seen.display()),
    );
    let result = read(&ProcessReader::new(program), &["pw1".into(), "pw2".into()])
        .await
        .unwrap();
    assert_eq!(result.outcome.id(), "locked");
    let request: serde_json::Value = serde_json::from_slice(&std::fs::read(seen).unwrap()).unwrap();
    assert_eq!(
        request,
        serde_json::json!({"version":2,"file_path":"/in/file.pdf","passwords":["pw1","pw2"],"decrypted_out":"/out/copy"})
    );
}

#[tokio::test]
async fn a_non_zero_exit_is_a_crash_with_its_code() {
    let dir = tempfile::tempdir().unwrap();
    let program = script(dir.path(), "cat > /dev/null\necho 'boom' >&2\nexit 3");
    let error = read(&ProcessReader::new(program), &[]).await.unwrap_err();
    assert!(matches!(error, ReaderError::Crashed(Some(3))), "{error:?}");
    assert_eq!(error.code(), "reader_crashed");
}

#[tokio::test]
async fn unreadable_output_is_malformed() {
    let dir = tempfile::tempdir().unwrap();
    let program = script(dir.path(), "cat > /dev/null\necho 'not json'");
    let error = read(&ProcessReader::new(program), &[]).await.unwrap_err();
    assert!(matches!(error, ReaderError::Malformed), "{error:?}");
}

#[tokio::test]
async fn a_slow_helper_is_killed_at_the_timeout() {
    let dir = tempfile::tempdir().unwrap();
    let pid_file = dir.path().join("pid");
    let program = script(
        dir.path(),
        &format!("echo $$ > '{}'\nexec sleep 30", pid_file.display()),
    );
    let reader = ProcessReader::new(program).with_timeout(Duration::from_secs(3));
    let started = std::time::Instant::now();
    let error = read(&reader, &[]).await.unwrap_err();
    assert!(matches!(error, ReaderError::Timeout), "{error:?}");
    assert!(started.elapsed() < Duration::from_secs(10));
    let pid = std::fs::read_to_string(pid_file).unwrap().trim().to_owned();
    tokio::time::sleep(Duration::from_millis(300)).await;
    let alive = std::process::Command::new("kill")
        .args(["-0", &pid])
        .status()
        .unwrap()
        .success();
    assert!(!alive, "helper {pid} should have been killed");
}

#[tokio::test]
async fn a_missing_program_cannot_be_started() {
    let error = read(&ProcessReader::new("/nonexistent/statement-reader"), &[])
        .await
        .unwrap_err();
    assert!(matches!(error, ReaderError::Spawn(_)), "{error:?}");
    assert_eq!(error.code(), "reader_unavailable");
}

#[tokio::test]
async fn the_request_id_reaches_the_helper_in_its_environment() {
    let dir = tempfile::tempdir().unwrap();
    let seen = dir.path().join("request-id");
    let program = script(
        dir.path(),
        &format!(
            "cat > /dev/null\nprintf %s \"$STATEMENT_READER_REQUEST_ID\" > '{}'\necho '{LOCKED}'",
            seen.display()
        ),
    );
    read(&ProcessReader::new(program), &[]).await.unwrap();
    assert_eq!(std::fs::read_to_string(seen).unwrap(), "req-42");
}

#[tokio::test]
async fn without_a_request_id_the_helper_gets_none() {
    let dir = tempfile::tempdir().unwrap();
    let seen = dir.path().join("request-id");
    let program = script(
        dir.path(),
        &format!(
            "cat > /dev/null\nprintf %s \"${{STATEMENT_READER_REQUEST_ID-unset}}\" > '{}'\necho '{LOCKED}'",
            seen.display()
        ),
    );
    ProcessReader::new(program)
        .read(Path::new("/f"), &[], Path::new("/o"), None)
        .await
        .unwrap();
    assert_eq!(std::fs::read_to_string(seen).unwrap(), "unset");
}
