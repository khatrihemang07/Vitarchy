//! Reads everything a Sync needs from Toggl, before anything is written, so a
//! failed call leaves the stored data untouched.

use chrono::{DateTime, Utc};

use super::window::window;
use crate::{
    repo::connection::Connection,
    toggl::{
        api::TogglApi,
        error::TogglError,
        types::{Raw, TogglClient, TogglEntry, TogglProject, TogglTag},
    },
};

/// Toggl's answers for one Sync.
pub struct Fetched {
    /// Time Entries in the requested window.
    pub entries: Vec<Raw<TogglEntry>>,
    /// All projects.
    pub projects: Vec<Raw<TogglProject>>,
    /// All clients.
    pub clients: Vec<Raw<TogglClient>>,
    /// All tags.
    pub tags: Vec<Raw<TogglTag>>,
}

/// Makes the four calls one after another (Toggl rate-limits per second).
pub async fn fetch(
    api: &dyn TogglApi,
    connection: &Connection,
    now: DateTime<Utc>,
) -> Result<Fetched, TogglError> {
    let wid = connection.workspace_id;
    Ok(Fetched {
        entries: api
            .time_entries(&window(connection.sync_cursor, now))
            .await?,
        projects: api.projects(wid).await?,
        clients: api.clients(wid).await?,
        tags: api.tags(wid).await?,
    })
}
