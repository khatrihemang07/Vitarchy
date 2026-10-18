//! Active time by app, window title and site.

use serde::Serialize;
use utoipa::ToSchema;

/// Active time in one window title.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct TitleUsage {
    /// The window title.
    pub title: String,
    /// Active seconds.
    pub active_secs: i64,
}

/// Active time on one site.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct SiteUsage {
    /// The site's domain.
    pub domain: String,
    /// Active seconds.
    pub active_secs: i64,
}

/// Active time in one app.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct AppUsage {
    /// The app.
    pub app: String,
    /// Active seconds.
    pub active_secs: i64,
    /// Share (0 to 1) of all Active time.
    pub share: f64,
    /// Its most used window titles, most Active time first (at most 10).
    pub titles: Vec<TitleUsage>,
    /// Its most used sites, most Active time first.
    pub sites: Vec<SiteUsage>,
}
