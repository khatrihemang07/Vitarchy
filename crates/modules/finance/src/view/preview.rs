//! Builds a [`StatementPreview`] from a stored Statement and what the database knows now.

use std::collections::HashMap;

use sqlx::PgConnection;

use super::{
    model::{DiagnosticsView, PeriodView, SectionPreview, StatementPreview},
    section::{self, Lookup, Matcher},
    summary::parse_result,
};
use crate::{
    reader::ReaderResult,
    repo::{
        accounts, instrument_accounts, printings, statement_accounts, statement_files,
        statements::StatementRow, transactions,
    },
};

/// The preview of `row`. `duplicate_file` is true when an upload repeated this stored file.
///
/// # Errors
/// The database failed.
pub async fn build(
    db: &mut PgConnection,
    row: &StatementRow,
    duplicate_file: bool,
) -> Result<StatementPreview, sqlx::Error> {
    let result = parse_result(row);
    let sections = match &result {
        Some(result) => sections(db, row.id, result).await?,
        None => vec![],
    };
    let diagnostics = result.as_ref().map(|r| &r.diagnostics);
    Ok(StatementPreview {
        id: row.id,
        file_name: row.file_name.clone(),
        format: row.format.clone(),
        kind: row.file_kind(),
        outcome: row.outcome.clone(),
        status: row.status.clone(),
        duplicate_file,
        diagnostics: DiagnosticsView {
            stage: diagnostics.and_then(|d| d.stage.clone()),
            code: diagnostics.and_then(|d| d.code.clone()),
            pages: diagnostics.and_then(|d| d.pages),
            rows: diagnostics.and_then(|d| d.rows),
        },
        period: overall_period(&sections),
        encrypted: result.as_ref().is_some_and(|r| r.encrypted),
        unlocked_with_label: statement_files::unlocked_with_label(&mut *db, row.id).await?,
        uploaded_at: row.uploaded_at,
        sections,
    })
}

async fn sections(
    db: &mut PgConnection,
    statement_id: i64,
    result: &ReaderResult,
) -> Result<Vec<SectionPreview>, sqlx::Error> {
    let slots = statement_accounts::list(&mut *db, statement_id).await?;
    let known = accounts::identities(&mut *db).await?;
    let instruments = instrument_accounts::all(&mut *db).await?;
    let matcher = Matcher {
        kind: result.kind,
        known: &known,
        instruments: &instruments,
    };
    let names: HashMap<i64, String> = accounts::list(&mut *db)
        .await?
        .into_iter()
        .map(|a| (a.id, a.name))
        .collect();
    let mut stored = HashMap::new();
    let mut printed = HashMap::new();
    for (slot, statement) in slots.iter().zip(&result.statements) {
        if let Some(id) = matcher.resolve(slot, statement).single()
            && let std::collections::hash_map::Entry::Vacant(entry) = stored.entry(id)
        {
            entry.insert(transactions::stored_keys(&mut *db, id).await?);
            printed.extend(printings::for_account(&mut *db, id).await?);
        }
    }
    let lookup = Lookup {
        names: &names,
        matcher,
        stored: &stored,
        printings: &printed,
        statement_id,
    };
    Ok(slots
        .iter()
        .zip(&result.statements)
        .map(|(slot, statement)| section::build(slot, statement, &lookup))
        .collect())
}

fn overall_period(sections: &[SectionPreview]) -> PeriodView {
    PeriodView {
        start: sections.iter().filter_map(|s| s.period.start).min(),
        end: sections.iter().filter_map(|s| s.period.end).max(),
    }
}
