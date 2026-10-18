//! `GET`, `POST` and `DELETE /api/finance/statement-passwords`: write-only; the secret is
//! never returned.

use std::fmt;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use vitarchy_core::{ApiError, ApiJson, AppState};

use crate::{
    error::FinanceError,
    problem::{ProblemBody, Refusal},
    repo::passwords::{self, PasswordLabel},
};

/// A saved password as listed: never its secret.
#[derive(Serialize, ToSchema)]
pub struct PasswordItem {
    /// Password id.
    pub id: i64,
    /// The name the person gave it.
    pub label: String,
    /// When it was saved.
    pub created_at: DateTime<Utc>,
    /// When it last opened a file.
    pub last_used_at: Option<DateTime<Utc>>,
}

impl From<PasswordLabel> for PasswordItem {
    fn from(p: PasswordLabel) -> Self {
        Self {
            id: p.id,
            label: p.label,
            created_at: p.created_at,
            last_used_at: p.last_used_at,
        }
    }
}

/// Body of `POST /api/finance/statement-passwords`.
#[derive(Deserialize, ToSchema)]
pub struct NewPassword {
    /// A name to recognise it by; trimmed, not empty.
    pub label: String,
    /// The password; trimmed, not empty. Stored, never returned.
    pub secret: String,
}

impl fmt::Debug for NewPassword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NewPassword")
            .field("label", &self.label)
            .field("secret", &"[hidden]")
            .finish()
    }
}

/// The saved passwords, oldest first, without their secrets.
#[utoipa::path(
    get,
    path = "/statement-passwords",
    tag = "finance",
    responses((status = 200, description = "The saved passwords", body = Vec<PasswordItem>))
)]
#[tracing::instrument(skip_all)]
pub async fn list_passwords(
    State(state): State<AppState>,
) -> Result<Json<Vec<PasswordItem>>, FinanceError> {
    let rows = passwords::labels(&state.pool).await?;
    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

/// Saves a Statement password. Locked files are tried with every saved password.
#[utoipa::path(
    post,
    path = "/statement-passwords",
    tag = "finance",
    request_body = NewPassword,
    responses(
        (status = 201, description = "Saved", body = PasswordItem),
        (status = 400, description = "Empty label or password (`empty_name`, `empty_secret`)", body = ProblemBody),
    )
)]
#[tracing::instrument(skip_all)]
pub async fn post_password(
    State(state): State<AppState>,
    ApiJson(body): ApiJson<NewPassword>,
) -> Result<(StatusCode, Json<PasswordItem>), FinanceError> {
    let (label, secret) = (body.label.trim(), body.secret.trim());
    if label.is_empty() {
        return Err(Refusal::EMPTY_NAME.into());
    }
    if secret.is_empty() {
        return Err(Refusal::EMPTY_SECRET.into());
    }
    let id = passwords::insert(&state.pool, label, secret).await?;
    let saved = passwords::labels(&state.pool)
        .await?
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| ApiError::NotFound("password vanished".to_owned()))?;
    tracing::info!(password_id = id, "statement password saved");
    Ok((StatusCode::CREATED, Json(saved.into())))
}

/// Forgets a saved password.
#[utoipa::path(
    delete,
    path = "/statement-passwords/{id}",
    tag = "finance",
    params(("id" = i64, Path, description = "Password id")),
    responses(
        (status = 204, description = "Forgotten"),
        (status = 404, description = "No such password"),
    )
)]
#[tracing::instrument(skip_all, fields(password_id = id))]
pub async fn delete_password(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, FinanceError> {
    if !passwords::delete(&state.pool, id).await? {
        return Err(ApiError::NotFound("no such password".to_owned()).into());
    }
    tracing::info!("statement password deleted");
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::NewPassword;

    #[test]
    fn debug_never_shows_the_secret() {
        let body = NewPassword {
            label: "Main".to_owned(),
            secret: "hunter2".to_owned(),
        };
        let shown = format!("{body:?}");
        assert!(!shown.contains("hunter2"), "{shown}");
        assert!(shown.contains("Main"));
    }
}
