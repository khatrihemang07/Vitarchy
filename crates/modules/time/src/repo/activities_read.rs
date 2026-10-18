//! `time.activities`, read back as [`Activity`] for the Timeline to build from.

use chrono::{DateTime, Utc};
use sqlx::PgExecutor;

use crate::{
    activity::{Activity, Extras, Source},
    timeline::TimeRange,
};

/// One stored row, before it is turned back into an [`Activity`]. `raw` is left in the
/// database: nothing that reads Activity back needs it.
#[derive(sqlx::FromRow)]
struct StoredActivity {
    source: String,
    source_id: String,
    client_id: Option<String>,
    app: String,
    title: String,
    url: Option<String>,
    start_at: DateTime<Utc>,
    end_at: DateTime<Utc>,
    idle: Option<bool>,
    idle_secs: Option<f64>,
}

impl StoredActivity {
    /// `None` for a Source this build does not know, which the table's check prevents.
    fn into_activity(self) -> Option<Activity> {
        Some(Activity {
            source: Source::from_id(&self.source)?,
            source_id: self.source_id,
            app: self.app,
            title: self.title,
            start_at: self.start_at,
            end_at: self.end_at,
            raw: serde_json::Value::Null,
            extras: Extras {
                client_id: self.client_id,
                idle: self.idle,
                url: self.url,
                idle_secs: self.idle_secs,
            },
        })
    }
}

/// Every Source's Activity that overlaps `range`, oldest first. `raw` is not loaded (it is
/// `Null`).
pub async fn overlapping<'e>(
    executor: impl PgExecutor<'e>,
    range: TimeRange,
) -> Result<Vec<Activity>, sqlx::Error> {
    let rows: Vec<StoredActivity> = sqlx::query_as(
        "SELECT source, source_id, client_id::text AS client_id, app, title, url, start_at, end_at,
                idle, idle_secs
         FROM time.activities WHERE start_at < $2 AND end_at > $1 ORDER BY start_at, source, source_id",
    )
    .bind(range.from)
    .bind(range.to)
    .fetch_all(executor)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(StoredActivity::into_activity)
        .collect())
}
