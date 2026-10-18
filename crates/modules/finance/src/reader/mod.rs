//! Reading a file into a [`ReaderResult`]. The server never parses Statements itself: it
//! runs the statement-reader helper (ADR 0012) behind the [`StatementReader`] seam.

mod availability;
pub mod contract;
mod error;
mod process;
mod request;
pub mod testing;

use std::path::Path;

use async_trait::async_trait;

pub use contract::ReaderResult;
pub use error::ReaderError;
pub use process::ProcessReader;
pub use request::Request;

/// Something that can read one uploaded file.
#[async_trait]
pub trait StatementReader: Send + Sync {
    /// Reads `file`, trying each of `passwords` if it is locked. When the file was encrypted
    /// and unlocked, a decrypted copy is written to `decrypted_out`. `request_id` is the server
    /// request this read belongs to; the helper tags its log lines with it.
    ///
    /// # Errors
    /// The helper could not run, crashed, timed out or answered something unreadable.
    async fn read(
        &self,
        file: &Path,
        passwords: &[String],
        decrypted_out: &Path,
        request_id: Option<&str>,
    ) -> Result<ReaderResult, ReaderError>;

    /// Whether the reader can run at all (its program exists). Fakes are always available.
    async fn available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod contract_tests;
