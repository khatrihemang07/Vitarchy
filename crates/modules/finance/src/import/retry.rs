//! Read again for an unconfirmed Statement: read the stored file afresh with the passwords
//! saved now, replacing its preview. This is how a locked file opens.

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
    files::FileStore,
    reader::StatementReader,
    repo::{
        passwords, runs, statement_accounts, statement_files,
        statements::{self, StatementRow},
    },
};

/// Re-reads an unconfirmed Statement in place, with the passwords saved now.
///
/// # Errors
/// The stored file is missing or the database failed.
pub async fn retry(
    pool: &PgPool,
    reader: &dyn StatementReader,
    files: &FileStore,
    row: &StatementRow,
    request_id: Option<&str>,
) -> Result<(), FinanceError> {
    let started_at = Utc::now();
    let outcome = retry_inner(pool, reader, files, row, request_id).await;
    runs::record_outcome(
        pool,
        "read_again",
        Some(row.id),
        started_at,
        outcome.as_ref(),
    )
    .await;
    outcome.map(|_| ())
}

async fn retry_inner(
    pool: &PgPool,
    reader: &dyn StatementReader,
    files: &FileStore,
    row: &StatementRow,
    request_id: Option<&str>,
) -> Result<ReadCounts, FinanceError> {
    let saved = reading::saved_passwords(pool).await?;
    let Reading {
        result,
        decrypted,
        unlocked_with,
    } = reading::read_stored(reader, files, &row.file_sha256, &saved, request_id).await?;
    let kept_sha = match &decrypted {
        Some(bytes) => Some(files.put(bytes).await?),
        None => None,
    };
    let stored = StoredResult::of(&result);
    let mut tx = pool.begin().await?;
    statements::replace_result(
        &mut *tx,
        row.id,
        &stored.row(&row.file_sha256, &row.file_name),
    )
    .await?;
    statement_accounts::clear(&mut *tx, row.id).await?;
    if let Ok(read) = &result {
        propose(&mut tx, row.id, read.kind, &read.statements).await?;
    }
    if let (Some(sha), Some((password_id, label))) = (&kept_sha, &unlocked_with) {
        statement_files::record_unlock(&mut *tx, row.id, sha, &row.file_sha256, Some(label))
            .await?;
        passwords::touch(&mut *tx, *password_id).await?;
    }
    tx.commit().await?;
    if kept_sha.is_some() && !statements::file_in_use(pool, &row.file_sha256).await? {
        files.remove(&row.file_sha256).await?;
    }
    Ok(result.as_ref().map(ReadCounts::of).unwrap_or_default())
}
