//! The content-addressed store of Statement originals (ADR 0013): each file lives at
//! `<root>/<sha256>`, so the same bytes are stored once and a path never changes.

use std::{
    ffi::OsString,
    fmt::Write as _,
    io,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

/// Environment variable naming the store's folder.
pub const DIR_ENV: &str = "FINANCE_STATEMENTS_DIR";
/// Where the store lives under `HOME` when [`DIR_ENV`] is unset.
const DEFAULT_RELATIVE: &str = "Library/Application Support/Vitarchy/statements";
/// Set by `scripts/with-env.sh` for stage: stage must never fall back to the real folder.
const STAGE_ENV: &str = "VITARCHY_STAGE";

/// A folder of files named by their SHA-256.
#[derive(Debug, Clone)]
pub struct FileStore {
    root: PathBuf,
}

impl FileStore {
    /// A store rooted at `root` (created on first write).
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The store at `FINANCE_STATEMENTS_DIR`; else, on stage, a throwaway folder in the
    /// system temp dir; else under `HOME` like the other Vitarchy data.
    #[must_use]
    pub fn from_env() -> Self {
        Self::resolve(
            std::env::var_os(DIR_ENV),
            std::env::var_os("HOME").map(PathBuf::from).as_deref(),
            std::env::var_os(STAGE_ENV).is_some(),
        )
    }

    /// The folder choice behind [`Self::from_env`], with its inputs passed in.
    #[must_use]
    pub fn resolve(dir: Option<OsString>, home: Option<&Path>, stage: bool) -> Self {
        match (dir.filter(|dir| !dir.is_empty()), home) {
            (Some(dir), _) => Self::new(dir),
            (None, _) if stage => Self::new(std::env::temp_dir().join("vitarchy-stage-statements")),
            (None, Some(home)) => Self::new(home.join(DEFAULT_RELATIVE)),
            (None, None) => Self::new("statements"),
        }
    }

    /// Stores `bytes` and returns their SHA-256 (lower-case hex). Storing the same bytes
    /// again changes nothing. The file appears whole or not at all: it is written beside
    /// its final name, then renamed.
    ///
    /// # Errors
    /// The folder or file could not be written.
    pub async fn put(&self, bytes: &[u8]) -> io::Result<String> {
        let sha = sha256_hex(bytes);
        let target = self.path(&sha);
        if tokio::fs::try_exists(&target).await? {
            return Ok(sha);
        }
        tokio::fs::create_dir_all(&self.root).await?;
        let partial = self.root.join(format!("{sha}.new"));
        tokio::fs::write(&partial, bytes).await?;
        tokio::fs::rename(&partial, &target).await?;
        Ok(sha)
    }

    /// Where the file with this hash is (or would be) stored.
    #[must_use]
    pub fn path(&self, sha: &str) -> PathBuf {
        self.root.join(sha)
    }

    /// Where a decrypted copy is written while a file is read, beside the original.
    #[must_use]
    pub fn scratch_path(&self, sha: &str) -> PathBuf {
        self.root.join(format!("{sha}.decrypted"))
    }

    /// Whether the file is stored.
    ///
    /// # Errors
    /// The folder could not be inspected.
    pub async fn contains(&self, sha: &str) -> io::Result<bool> {
        tokio::fs::try_exists(self.path(sha)).await
    }

    /// Deletes the file; already gone is fine.
    ///
    /// # Errors
    /// The file exists but could not be removed.
    pub async fn remove(&self, sha: &str) -> io::Result<()> {
        match tokio::fs::remove_file(self.path(sha)).await {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        }
    }

    /// The folder holding the files.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}

/// Lower-case hex SHA-256 of `bytes`.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

#[cfg(test)]
mod tests;
