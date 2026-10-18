//! The read-only half of Read again: re-read the stored original and report what would
//! change, writing nothing.

use chrono::Utc;
use sqlx::PgPool;

use super::{
    read_again_diff::{Diff, compute},
    read_counts::ReadCounts,
    reading,
};
use crate::{
    error::FinanceError,
    files::FileStore,
    problem::Refusal,
    reader::{
        ReaderResult, StatementReader,
        contract::{Kind, Outcome},
    },
    repo::{runs, statements::StatementRow},
};

/// Reads the stored original of a confirmed Statement and compares it with what is stored.
///
/// # Errors
/// The engine can no longer read the file as before, or the file or database failed.
pub async fn preview(
    pool: &PgPool,
    reader: &dyn StatementReader,
    files: &FileStore,
    row: &StatementRow,
    request_id: Option<&str>,
) -> Result<(ReaderResult, Diff), FinanceError> {
    let started_at = Utc::now();
    let outcome = async {
        let result = fresh(pool, reader, files, row, request_id).await?;
        let mut db = pool.acquire().await?;
        let diff = compute(&mut db, row.id, &result).await?;
        Ok((result, diff))
    }
    .await;
    let counts = outcome.as_ref().map(|(result, _)| ReadCounts::of(result));
    runs::record_outcome(
        pool,
        "read_again",
        Some(row.id),
        started_at,
        counts.as_ref().map_err(|e| *e),
    )
    .await;
    outcome
}

/// Reads the stored original and checks it is still the kind of file it was.
pub(super) async fn fresh(
    pool: &PgPool,
    reader: &dyn StatementReader,
    files: &FileStore,
    row: &StatementRow,
    request_id: Option<&str>,
) -> Result<ReaderResult, FinanceError> {
    let saved = reading::saved_passwords(pool).await?;
    let reading = reading::read_stored(reader, files, &row.file_sha256, &saved, request_id).await?;
    let result = reading.result?;
    let same_kind = result.kind.map(Kind::id) == row.kind.as_deref();
    if result.outcome != Outcome::Read || !same_kind {
        return Err(Refusal::NOT_READABLE.into());
    }
    Ok(result)
}
