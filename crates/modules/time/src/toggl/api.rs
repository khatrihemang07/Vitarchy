//! The seam between Vitarchy and Toggl. Production uses [`super::client::HttpToggl`];
//! tests plug in a fake, so no test ever calls the real Toggl.

use std::sync::Arc;

use async_trait::async_trait;

use super::{
    error::TogglError,
    types::{EntryQuery, Me, Raw, TogglClient, TogglEntry, TogglProject, TogglTag, Workspace},
};

/// What Vitarchy asks of Toggl, already bound to one person's token.
#[async_trait]
pub trait TogglApi: Send + Sync {
    /// Validates the token and names the default workspace.
    async fn me(&self) -> Result<Me, TogglError>;
    /// A workspace's details.
    async fn workspace(&self, workspace_id: i64) -> Result<Workspace, TogglError>;
    /// The person's Time Entries matching `query` (deleted ones included for `Since`).
    async fn time_entries(&self, query: &EntryQuery) -> Result<Vec<Raw<TogglEntry>>, TogglError>;
    /// Every project in the workspace, archived ones too.
    async fn projects(&self, workspace_id: i64) -> Result<Vec<Raw<TogglProject>>, TogglError>;
    /// Every client in the workspace.
    async fn clients(&self, workspace_id: i64) -> Result<Vec<Raw<TogglClient>>, TogglError>;
    /// Every tag in the workspace.
    async fn tags(&self, workspace_id: i64) -> Result<Vec<Raw<TogglTag>>, TogglError>;
}

/// Builds a [`TogglApi`] for a token. Routes receive it as an axum `Extension`,
/// which is how tests swap in a fake without core's `AppState` knowing about Toggl.
pub type TogglFactory = Arc<dyn Fn(&str) -> Arc<dyn TogglApi> + Send + Sync>;
