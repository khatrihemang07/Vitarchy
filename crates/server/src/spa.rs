//! Serves the built web app (`web/dist`) from the same port as the API.
//!
//! Used by `make start`, where the Rust server is the whole app. In dev the
//! Vite server serves the UI and this module is not mounted.

use std::path::{Path, PathBuf};

use axum::{
    Router,
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

/// The web build is missing or incomplete.
#[derive(Debug, Error)]
#[error("WEB_DIST {} has no index.html; build the web app first (`make web-build`)", .0.display())]
pub struct NotBuilt(PathBuf);

/// Confirms `dist` holds a built web app.
///
/// # Errors
/// [`NotBuilt`] when `dist/index.html` is missing, so the server fails at
/// startup with a clear message instead of answering 404 to every page.
pub fn ensure_built(dist: &Path) -> Result<(), NotBuilt> {
    if dist.join("index.html").is_file() {
        Ok(())
    } else {
        Err(NotBuilt(dist.to_path_buf()))
    }
}

/// Whether `path` looks like a client-side route rather than a file request:
/// no file extension, and not under the build's `assets/` directory.
fn is_navigation(path: &str) -> bool {
    let path = path.trim_start_matches('/');
    let first = path.split('/').next().unwrap_or("");
    let last = path.rsplit('/').next().unwrap_or("");
    first != "assets" && !last.contains('.')
}

/// Serves `index.html` for navigation-like paths and 404 for anything else, so a
/// missing script or image is an honest error instead of HTML in disguise.
async fn index_or_not_found(index: ServeFile, req: Request) -> Response {
    if !is_navigation(req.uri().path()) {
        return StatusCode::NOT_FOUND.into_response();
    }
    match index.oneshot(req).await {
        Ok(res) => res.into_response(),
        Err(never) => match never {},
    }
}

/// Makes `router` answer every request it has no route for with a file from
/// `dist`. Navigation-like paths (such as `/settings/general`) fall back to
/// `index.html` so client-side routes survive a reload; missing files and
/// `/assets/*` are 404. API routes are matched first, and unknown `/api/*`
/// paths are caught before this fallback (see `app`).
pub fn mount(router: Router, dist: &Path) -> Router {
    let index = ServeFile::new(dist.join("index.html"));
    let fallback = Router::new().fallback(move |req| index_or_not_found(index.clone(), req));
    router.fallback_service(ServeDir::new(dist).fallback(fallback))
}
