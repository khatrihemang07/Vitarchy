//! One uploaded file: store it, read it, keep the decrypted copy if it was locked, and save
//! an unconfirmed Statement with an Account proposal per section.

use std::time::Instant;

use chrono::Utc;
use sqlx::PgPool;

use super::{
    proposal::propose,
    read_counts::ReadCounts,
    reading::{self, Reading},
    stored_result::StoredResult,
};
use crate::{
    error::FinanceError,
    files::{FileStore, sha256_hex},
    reader::{ReaderError, StatementReader},
    repo::{passwords, runs, statement_files, statements},
};

/// The Statement an upload became.
pub struct Imported {
    /// Its id.
    pub id: i64,
    /// True when the same file was already stored and nothing was read.
    pub duplicate: bool,
}

/// A saved Statement and whether the reader managed to read it.
struct Saved {
    imported: Imported,
    read: Result<ReadCounts, ReaderError>,
}

/// Imports one file.
///
/// # Errors
/// The file store or database failed. A reader that fails is not an error here: the file is
/// kept as a `crashed` Statement so it can be read again.
pub async fn import_file(
    pool: &PgPool,
    reader: &dyn StatementReader,
    files: &FileStore,
    file_name: &str,
    bytes: &[u8],
    request_id: Option<&str>,
) -> Result<Imported, FinanceError> {
    let started_at = Utc::now();
    let timer = Instant::now();
    let upload_sha = sha256_hex(bytes);
    if let Some(existing) = statement_files::find_by_sha(pool, &upload_sha).await? {
        tracing::info!(statement_id = existing.id, "upload repeats a stored file");
        return Ok(Imported {
            id: existing.id,
            duplicate: true,
        });
    }
    files.put(bytes).await?;
    let saved = read_and_save(pool, reader, files, file_name, &upload_sha, request_id).await;
    let duration_ms = u64::try_from(timer.elapsed().as_millis()).unwrap_or(u64::MAX);
    match saved {
        Ok(Saved { imported, read }) => {
            let run = read.map_err(FinanceError::from);
            runs::record_outcome(pool, "read", Some(imported.id), started_at, run.as_ref()).await;
            tracing::info!(
                statement_id = imported.id,
                ok = run.is_ok(),
                duration_ms,
                "file read"
            );
            Ok(imported)
        }
        Err(error) => {
            runs::record_outcome::<ReadCounts>(pool, "read", None, started_at, Err(&error)).await;
            Err(error)
        }
    }
}

async fn read_and_save(
    pool: &PgPool,
    reader: &dyn StatementReader,
    files: &FileStore,
    file_name: &str,
    upload_sha: &str,
    request_id: Option<&str>,
) -> Result<Saved, FinanceError> {
    let saved = reading::saved_passwords(pool).await?;
    let Reading {
        result,
        decrypted,
        unlocked_with,
    } = reading::read_stored(reader, files, upload_sha, &saved, request_id).await?;
    let kept_sha = match &decrypted {
        Some(bytes) => files.put(bytes).await?,
        None => upload_sha.to_owned(),
    };
    if decrypted.is_some()
        && let Some(existing) = statement_files::find_by_sha(pool, &kept_sha).await?
    {
        files.remove(upload_sha).await?;
        return Ok(Saved {
            imported: Imported {
                id: existing.id,
                duplicate: true,
            },
            read: Ok(ReadCounts::default()),
        });
    }
    let stored = StoredResult::of(&result);
    let mut tx = pool.begin().await?;
    let id = statements::insert(&mut *tx, &stored.row(&kept_sha, file_name)).await?;
    if let Some((password_id, label)) = &unlocked_with {
        statement_files::record_unlock(&mut *tx, id, &kept_sha, upload_sha, Some(label)).await?;
        passwords::touch(&mut *tx, *password_id).await?;
    }
    if let Ok(read) = &result {
        propose(&mut tx, id, read.kind, &read.statements).await?;
    }
    tx.commit().await?;
    if decrypted.is_some() {
        files.remove(upload_sha).await?;
    }
    Ok(Saved {
        imported: Imported {
            id,
            duplicate: false,
        },
        read: result.map(|r| ReadCounts::of(&r)),
    })
}
