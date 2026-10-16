//! The real Toggl API v9 client (reqwest, Basic auth `<token>:api_token`).

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::{
    api::{TogglApi, TogglFactory},
    error::TogglError,
    types::{EntryQuery, Me, Raw, TogglClient, TogglEntry, TogglProject, TogglTag, Workspace},
};

/// Toggl's production API root.
pub const TOGGL_BASE_URL: &str = "https://api.track.toggl.com/api/v9";
const QUOTA_HEADER: &str = "x-toggl-quota-resets-in";
const TIMEOUT: Duration = Duration::from_secs(30);
/// Projects per request (Toggl's maximum); a shorter page is the last one.
const PROJECTS_PER_PAGE: usize = 200;
/// A stop for a server that never returns a short page: 200 pages is 40,000 projects.
const MAX_PROJECT_PAGES: u32 = 200;

/// A [`TogglApi`] that calls Toggl over HTTPS with one person's token.
pub struct HttpToggl {
    http: reqwest::Client,
    base_url: String,
    token: String,
}

impl HttpToggl {
    /// Binds a client to `token`. `base_url` is Toggl's root, without a trailing slash.
    ///
    /// # Panics
    /// If the TLS backend cannot initialise, which no input can cause.
    #[must_use]
    pub fn new(token: &str, base_url: &str) -> Self {
        let http = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .expect("reqwest client builds with static settings");
        Self {
            http,
            base_url: base_url.to_owned(),
            token: token.to_owned(),
        }
    }

    /// The factory production wiring uses: real Toggl for any token.
    #[must_use]
    pub fn factory() -> TogglFactory {
        Arc::new(|token| Arc::new(Self::new(token, TOGGL_BASE_URL)))
    }

    async fn get(&self, path: &str, query: &[(&str, String)]) -> Result<Value, TogglError> {
        let response = self
            .http
            .get(format!("{}{path}", self.base_url))
            .basic_auth(&self.token, Some("api_token"))
            .query(query)
            .send()
            .await
            // The error text can echo the URL; the caller only needs the class.
            .map_err(|_| TogglError::Unreachable)?;
        let resets_in = response
            .headers()
            .get(QUOTA_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        if let Some(error) =
            TogglError::from_status(response.status().as_u16(), resets_in.as_deref())
        {
            return Err(error);
        }
        response.json().await.map_err(|_| TogglError::Malformed)
    }

    async fn get_typed<T: DeserializeOwned>(&self, path: &str) -> Result<T, TogglError> {
        serde_json::from_value(self.get(path, &[]).await?).map_err(|_| TogglError::Malformed)
    }

    async fn list<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Vec<Raw<T>>, TogglError> {
        Raw::parse_list(self.get(path, query).await?)
    }
}

#[async_trait]
impl TogglApi for HttpToggl {
    async fn me(&self) -> Result<Me, TogglError> {
        self.get_typed("/me").await
    }

    async fn workspace(&self, workspace_id: i64) -> Result<Workspace, TogglError> {
        self.get_typed(&format!("/workspaces/{workspace_id}")).await
    }

    async fn time_entries(&self, query: &EntryQuery) -> Result<Vec<Raw<TogglEntry>>, TogglError> {
        let params = match query {
            EntryQuery::Range {
                start_date,
                end_date,
            } => vec![
                ("start_date", start_date.to_string()),
                ("end_date", end_date.to_string()),
            ],
            EntryQuery::Since(unix) => vec![("since", unix.to_string())],
        };
        self.list("/me/time_entries", &params).await
    }

    async fn projects(&self, workspace_id: i64) -> Result<Vec<Raw<TogglProject>>, TogglError> {
        let path = format!("/workspaces/{workspace_id}/projects");
        let mut projects = Vec::new();
        for page in 1..=MAX_PROJECT_PAGES {
            let params = [
                ("active", "both".to_owned()),
                ("per_page", PROJECTS_PER_PAGE.to_string()),
                ("page", page.to_string()),
            ];
            let batch: Vec<Raw<TogglProject>> = self.list(&path, &params).await?;
            let last = batch.len() < PROJECTS_PER_PAGE;
            projects.extend(batch);
            if last {
                return Ok(projects);
            }
        }
        // Never a short page: something is wrong, and a partial list must not look complete.
        Err(TogglError::Malformed)
    }

    // Toggl's clients and tags endpoints take no paging parameters (v9 docs): one request returns all.
    async fn clients(&self, workspace_id: i64) -> Result<Vec<Raw<TogglClient>>, TogglError> {
        self.list(
            &format!("/workspaces/{workspace_id}/clients"),
            &[("status", "both".to_owned())],
        )
        .await
    }

    async fn tags(&self, workspace_id: i64) -> Result<Vec<Raw<TogglTag>>, TogglError> {
        self.list(&format!("/workspaces/{workspace_id}/tags"), &[])
            .await
    }
}
