//! `time.mirror_owner`: whose Toggl data the mirror holds (outlives the connection).

use sqlx::PgConnection;

use crate::sync::account::Account;

/// The recorded owner, locked until the transaction ends so two connects cannot race.
pub async fn get_for_update(tx: &mut PgConnection) -> Result<Option<Account>, sqlx::Error> {
    let row: Option<(i64, i64)> =
        sqlx::query_as("SELECT toggl_user_id, workspace_id FROM time.mirror_owner FOR UPDATE")
            .fetch_optional(tx)
            .await?;
    Ok(row.map(|(user_id, workspace_id)| Account {
        user_id,
        workspace_id,
    }))
}

/// Records `account` as the mirror's owner.
pub async fn set(tx: &mut PgConnection, account: Account) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO time.mirror_owner (toggl_user_id, workspace_id) VALUES ($1, $2)
         ON CONFLICT (singleton) DO UPDATE SET
           toggl_user_id = EXCLUDED.toggl_user_id, workspace_id = EXCLUDED.workspace_id",
    )
    .bind(account.user_id)
    .bind(account.workspace_id)
    .execute(tx)
    .await?;
    Ok(())
}
