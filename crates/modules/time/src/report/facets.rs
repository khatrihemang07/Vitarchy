//! What the filter pickers can offer: only what has Time Entries, most recently used first.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

/// A project that has at least one Time Entry.
#[derive(Debug, Clone, Serialize, ToSchema, sqlx::FromRow)]
pub struct ProjectFacet {
    /// Toggl's id.
    pub id: i64,
    /// Name.
    pub name: String,
    /// Colour as Toggl stores it (e.g. `#06aaf5`), when set.
    pub color: Option<String>,
    /// False once the project was archived in Toggl.
    pub active: bool,
    /// The project's client, when it has one.
    pub client_id: Option<i64>,
    /// Start of the latest entry using it.
    pub last_used: DateTime<Utc>,
}

/// A tag that has at least one Time Entry.
#[derive(Debug, Clone, Serialize, ToSchema, sqlx::FromRow)]
pub struct TagFacet {
    /// The tag's name (entries carry names, not ids).
    pub name: String,
    /// Start of the latest entry using it.
    pub last_used: DateTime<Utc>,
}

/// A client with at least one Time Entry (through its projects).
#[derive(Debug, Clone, Serialize, ToSchema, sqlx::FromRow)]
pub struct ClientFacet {
    /// Toggl's id.
    pub id: i64,
    /// Name.
    pub name: String,
    /// True once the client was archived in Toggl.
    pub archived: bool,
    /// Start of the latest entry under it.
    pub last_used: DateTime<Utc>,
}

/// Body of `GET /api/time/facets`; each list is sorted by `last_used`, newest first.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct Facets {
    /// Projects with entries (archived ones included).
    pub projects: Vec<ProjectFacet>,
    /// Tags with entries.
    pub tags: Vec<TagFacet>,
    /// Clients with entries (archived ones included).
    pub clients: Vec<ClientFacet>,
}
