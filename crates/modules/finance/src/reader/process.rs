//! Runs the statement-reader helper as a child process, once per file.

use std::{
    ffi::OsString,
    path::Path,
    process::Stdio,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use tokio::{io::AsyncWriteExt, process::Command};

use super::{ReaderError, ReaderResult, Request, StatementReader};

/// Environment variable naming the helper's executable.
pub const READER_ENV: &str = "FINANCE_READER";
/// Used when [`READER_ENV`] is unset: the helper on `PATH`.
pub const DEFAULT_PROGRAM: &str = "statement-reader";
/// Environment variable carrying the server's request id to the helper, so its stderr lines
/// can be matched to the request (CONTRACT.md).
pub const REQUEST_ID_ENV: &str = "STATEMENT_READER_REQUEST_ID";
/// How long one file may take before the helper is killed.
pub const TIMEOUT: Duration = Duration::from_mins(2);

/// Spawns the helper: request JSON on stdin, result JSON on stdout.
#[derive(Debug, Clone)]
pub struct ProcessReader {
    program: OsString,
    timeout: Duration,
}

impl ProcessReader {
    /// A reader running `program`.
    #[must_use]
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            timeout: TIMEOUT,
        }
    }

    /// The program from `FINANCE_READER`, else `statement-reader` on `PATH`.
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(std::env::var_os(READER_ENV).unwrap_or_else(|| DEFAULT_PROGRAM.into()))
    }

    /// Replaces the time limit (tests use a short one).
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    async fn run(
        &self,
        body: Vec<u8>,
        request_id: Option<&str>,
    ) -> Result<std::process::Output, ReaderError> {
        let mut command = Command::new(&self.program);
        if let Some(id) = request_id {
            command.env(REQUEST_ID_ENV, id);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            // Dropping the future on timeout must not leave the helper running.
            .kill_on_drop(true)
            .spawn()
            .map_err(ReaderError::Spawn)?;
        let mut stdin = child.stdin.take().ok_or(ReaderError::Malformed)?;
        let exchange = async move {
            // A helper that exits early closes the pipe; its exit status tells the story.
            let _ = stdin.write_all(&body).await;
            drop(stdin);
            child.wait_with_output().await.map_err(ReaderError::Io)
        };
        tokio::time::timeout(self.timeout, exchange)
            .await
            .map_err(|_| ReaderError::Timeout)?
    }
}

#[async_trait]
impl StatementReader for ProcessReader {
    async fn read(
        &self,
        file: &Path,
        passwords: &[String],
        decrypted_out: &Path,
        request_id: Option<&str>,
    ) -> Result<ReaderResult, ReaderError> {
        let started = Instant::now();
        let body = serde_json::to_vec(&Request::new(file, passwords, decrypted_out))
            .map_err(|_| ReaderError::Malformed)?;
        let output = self.run(body, request_id).await?;
        // By contract stderr holds stage names, counts and timings only.
        for line in String::from_utf8_lossy(&output.stderr).lines() {
            tracing::debug!(reader_log = line, "statement reader");
        }
        let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        if !output.status.success() {
            tracing::warn!(exit = ?output.status.code(), duration_ms, "statement reader crashed");
            return Err(ReaderError::Crashed(output.status.code()));
        }
        let result = serde_json::from_slice(&output.stdout).map_err(|_| ReaderError::Malformed)?;
        tracing::info!(duration_ms, "statement reader finished");
        Ok(result)
    }

    async fn available(&self) -> bool {
        super::availability::program_exists(&self.program, std::env::var_os("PATH").as_deref())
    }
}
