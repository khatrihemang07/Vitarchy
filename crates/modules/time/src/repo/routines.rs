//! `time.routines` and `time.routine_steps`: the person's saved Routines.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};

use crate::analysis::routines::{MatchKind, Routine, RoutineDraft, RoutineError, Step};
use crate::error::TimeError;

#[derive(sqlx::FromRow)]
struct RoutineRow {
    id: i64,
    name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct StepRow {
    routine_id: i64,
    label: String,
    match_kind: String,
    match_value: String,
}

/// Every Routine with its Steps, by name.
pub async fn list(pool: &PgPool) -> Result<Vec<Routine>, sqlx::Error> {
    let rows: Vec<RoutineRow> = sqlx::query_as(
        "SELECT id, name, created_at, updated_at FROM time.routines ORDER BY lower(name), id",
    )
    .fetch_all(pool)
    .await?;
    let steps: Vec<StepRow> = sqlx::query_as(
        "SELECT routine_id, label, match_kind, match_value FROM time.routine_steps
         ORDER BY routine_id, position",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|row| assemble(row, &steps)).collect())
}

/// One Routine, or `None` when the id is unknown.
pub async fn get(pool: &PgPool, id: i64) -> Result<Option<Routine>, sqlx::Error> {
    let row: Option<RoutineRow> =
        sqlx::query_as("SELECT id, name, created_at, updated_at FROM time.routines WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;
    let Some(row) = row else { return Ok(None) };
    let steps: Vec<StepRow> = sqlx::query_as(
        "SELECT routine_id, label, match_kind, match_value FROM time.routine_steps
         WHERE routine_id = $1 ORDER BY position",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;
    Ok(Some(assemble(row, &steps)))
}

/// Stores a new Routine.
///
/// # Errors
/// [`RoutineError::NameTaken`] when another Routine has the name (ignoring case).
pub async fn create(pool: &PgPool, draft: &RoutineDraft) -> Result<i64, TimeError> {
    let mut tx = pool.begin().await?;
    let id: i64 = sqlx::query_scalar("INSERT INTO time.routines (name) VALUES ($1) RETURNING id")
        .bind(&draft.name)
        .fetch_one(&mut *tx)
        .await
        .map_err(name_taken)?;
    write_steps(&mut tx, id, &draft.steps).await?;
    tx.commit().await?;
    Ok(id)
}

/// Replaces a Routine's name and Steps.
///
/// # Errors
/// [`RoutineError::NotFound`] for an unknown id, [`RoutineError::NameTaken`] for a clashing name.
pub async fn replace(pool: &PgPool, id: i64, draft: &RoutineDraft) -> Result<(), TimeError> {
    let mut tx = pool.begin().await?;
    let changed =
        sqlx::query("UPDATE time.routines SET name = $2, updated_at = now() WHERE id = $1")
            .bind(id)
            .bind(&draft.name)
            .execute(&mut *tx)
            .await
            .map_err(name_taken)?
            .rows_affected();
    if changed == 0 {
        return Err(RoutineError::NotFound.into());
    }
    sqlx::query("DELETE FROM time.routine_steps WHERE routine_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    write_steps(&mut tx, id, &draft.steps).await?;
    tx.commit().await?;
    Ok(())
}

/// Deletes a Routine and its Steps; returns whether it existed.
pub async fn delete(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let done = sqlx::query("DELETE FROM time.routines WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(done.rows_affected() > 0)
}

async fn write_steps(
    tx: &mut Transaction<'_, Postgres>,
    id: i64,
    steps: &[Step],
) -> Result<(), sqlx::Error> {
    for (position, step) in steps.iter().enumerate() {
        sqlx::query(
            "INSERT INTO time.routine_steps (routine_id, position, label, match_kind, match_value)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id)
        .bind(i32::try_from(position).unwrap_or(i32::MAX))
        .bind(&step.label)
        .bind(step.kind.as_str())
        .bind(&step.value)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

/// A unique-name violation becomes [`RoutineError::NameTaken`]; anything else stays a database error.
fn name_taken(error: sqlx::Error) -> TimeError {
    match &error {
        sqlx::Error::Database(db) if db.is_unique_violation() => RoutineError::NameTaken.into(),
        _ => error.into(),
    }
}

fn assemble(row: RoutineRow, steps: &[StepRow]) -> Routine {
    Routine {
        id: row.id,
        name: row.name,
        steps: steps
            .iter()
            .filter(|s| s.routine_id == row.id)
            .filter_map(|s| {
                Some(Step {
                    label: s.label.clone(),
                    kind: MatchKind::parse(&s.match_kind)?,
                    value: s.match_value.clone(),
                })
            })
            .collect(),
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}
