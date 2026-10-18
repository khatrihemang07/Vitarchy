//! Running the reader over a stored file with the saved passwords, and keeping the
//! decrypted copy it may produce (ADR 0013).

use std::collections::HashMap;

use sqlx::PgPool;

use crate::{
    files::FileStore,
    reader::{ReaderError, ReaderResult, StatementReader},
    repo::passwords,
};

/// A saved Statement password. Deliberately not `Debug`: it holds the secret.
pub struct Saved {
    /// Row id.
    pub id: i64,
    /// The name the person gave it.
    pub label: String,
    /// The secret, handed only to the reader.
    pub secret: String,
}

/// The saved passwords in the order they are tried.
///
/// # Errors
/// The database failed.
pub async fn saved_passwords(pool: &PgPool) -> Result<Vec<Saved>, sqlx::Error> {
    let labels: HashMap<i64, String> = passwords::labels(pool)
        .await?
        .into_iter()
        .map(|p| (p.id, p.label))
        .collect();
    Ok(passwords::secrets(pool)
        .await?
        .into_iter()
        .filter_map(|p| {
            Some(Saved {
                label: labels.get(&p.id)?.clone(),
                id: p.id,
                secret: p.secret,
            })
        })
        .collect())
}

/// What one run of the reader produced.
pub struct Reading {
    /// The reader's answer, or why there is none.
    pub result: Result<ReaderResult, ReaderError>,
    /// The decrypted bytes, when the file was encrypted and a password opened it.
    pub decrypted: Option<Vec<u8>>,
    /// The saved password that opened it: its id and label.
    pub unlocked_with: Option<(i64, String)>,
}

/// Reads the stored file `sha` with every saved password.
///
/// # Errors
/// The stored file is missing or the scratch copy could not be read or removed.
pub async fn read_stored(
    reader: &dyn StatementReader,
    files: &FileStore,
    sha: &str,
    saved: &[Saved],
    request_id: Option<&str>,
) -> std::io::Result<Reading> {
    if !files.contains(sha).await? {
        return Err(std::io::ErrorKind::NotFound.into());
    }
    let scratch = files.scratch_path(sha);
    let secrets: Vec<String> = saved.iter().map(|p| p.secret.clone()).collect();
    let result = reader
        .read(&files.path(sha), &secrets, &scratch, request_id)
        .await;
    let unlocked = result
        .as_ref()
        .ok()
        .filter(|r| r.encrypted)
        .and_then(|r| r.unlocked_with)
        .and_then(|index| saved.get(index));
    let unlocked_with = unlocked.map(|p| (p.id, p.label.clone()));
    let decrypted = take_scratch(&scratch, unlocked_with.is_some()).await?;
    Ok(Reading {
        result,
        decrypted,
        unlocked_with,
    })
}

/// Reads and removes the scratch copy; without an unlock any copy is stale and just removed.
async fn take_scratch(scratch: &std::path::Path, keep: bool) -> std::io::Result<Option<Vec<u8>>> {
    let bytes = match tokio::fs::read(scratch).await {
        Ok(bytes) => keep.then_some(bytes),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    tokio::fs::remove_file(scratch).await?;
    Ok(bytes)
}
