//! A reader that returns canned results, for tests of everything built on the seam.

use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::Mutex,
};

use async_trait::async_trait;

use super::{ReaderError, ReaderResult, StatementReader};

/// One call the fake received.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    /// The file it was asked to read.
    pub file: PathBuf,
    /// How many passwords it was given (never the passwords).
    pub password_count: usize,
    /// The request id it was given.
    pub request_id: Option<String>,
}

/// Answers each `read` with the next canned result, then repeats the last one.
#[derive(Default)]
pub struct FakeReader {
    answers: Mutex<VecDeque<Result<ReaderResult, ReaderError>>>,
    calls: Mutex<Vec<Call>>,
    decrypted: Option<Vec<u8>>,
}

impl FakeReader {
    /// A fake that answers `results` in order.
    #[must_use]
    pub fn with_results(results: impl IntoIterator<Item = ReaderResult>) -> Self {
        Self::with_answers(results.into_iter().map(Ok))
    }

    /// A fake that answers `answers` in order, errors included.
    #[must_use]
    pub fn with_answers(
        answers: impl IntoIterator<Item = Result<ReaderResult, ReaderError>>,
    ) -> Self {
        Self {
            answers: Mutex::new(answers.into_iter().collect()),
            calls: Mutex::default(),
            decrypted: None,
        }
    }

    /// Makes the fake write `bytes` to `decrypted_out` whenever it answers that an encrypted
    /// file was unlocked, as the real helper does.
    #[must_use]
    pub fn writing_decrypted(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.decrypted = Some(bytes.into());
        self
    }

    /// The calls made so far.
    ///
    /// # Panics
    /// Never in practice: the lock is only held briefly.
    #[must_use]
    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("calls lock").clone()
    }
}

#[async_trait]
impl StatementReader for FakeReader {
    async fn read(
        &self,
        file: &Path,
        passwords: &[String],
        decrypted_out: &Path,
        request_id: Option<&str>,
    ) -> Result<ReaderResult, ReaderError> {
        self.calls.lock().expect("calls lock").push(Call {
            file: file.to_owned(),
            password_count: passwords.len(),
            request_id: request_id.map(str::to_owned),
        });
        let mut answers = self.answers.lock().expect("answers lock");
        let answer = if answers.len() > 1 {
            answers.pop_front()
        } else {
            answers.front().map(copy)
        };
        let answer = answer.unwrap_or(Err(ReaderError::Crashed(None)));
        if let (Ok(result), Some(bytes)) = (&answer, &self.decrypted)
            && result.encrypted
            && result.unlocked_with.is_some()
        {
            std::fs::write(decrypted_out, bytes).map_err(ReaderError::Io)?;
        }
        answer
    }
}

/// Repeats a canned answer; `io::Error` is not `Clone`, so it is rebuilt from its kind.
fn copy(answer: &Result<ReaderResult, ReaderError>) -> Result<ReaderResult, ReaderError> {
    let io = |e: &std::io::Error| std::io::Error::new(e.kind(), e.to_string());
    match answer {
        Ok(result) => Ok(result.clone()),
        Err(ReaderError::Spawn(e)) => Err(ReaderError::Spawn(io(e))),
        Err(ReaderError::Io(e)) => Err(ReaderError::Io(io(e))),
        Err(ReaderError::Timeout) => Err(ReaderError::Timeout),
        Err(ReaderError::Crashed(code)) => Err(ReaderError::Crashed(*code)),
        Err(ReaderError::Malformed) => Err(ReaderError::Malformed),
    }
}
