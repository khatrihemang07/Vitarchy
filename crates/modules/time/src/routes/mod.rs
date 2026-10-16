//! `/api/time`: connect Toggl, see the status, run a Sync, Import Activity, analyse, save Routines.
//!
//! **Injecting Toggl.** `AppState` (core) holds only the pool and must stay
//! domain-free, so the Toggl client reaches handlers as an axum `Extension`
//! holding a [`TogglFactory`]. [`router`] layers the real HTTP factory; tests
//! call [`router_with`] to layer a fake. Nothing in core or the server changes.

mod activity_status;
mod analysis;
mod connection;
mod import;
mod routines;
mod run_info;
mod status;
mod sync;

use axum::Extension;
use utoipa_axum::router::OpenApiRouter;
use vitarchy_core::AppState;

use crate::{
    activity::SnapshotFile,
    toggl::{api::TogglFactory, client::HttpToggl},
};

/// The production routes: talks to the real Toggl.
pub fn router() -> OpenApiRouter<AppState> {
    router_with(HttpToggl::factory(), SnapshotFile::from_env())
}

/// The routes with a chosen way of reaching Toggl and a chosen Snapshot file (tests pass fakes).
pub fn router_with(factory: TogglFactory, snapshot: SnapshotFile) -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(utoipa_axum::routes!(
            connection::put_connection,
            connection::delete_connection
        ))
        .routes(utoipa_axum::routes!(status::get_status))
        .routes(utoipa_axum::routes!(sync::post_sync))
        .routes(utoipa_axum::routes!(import::post_import))
        .routes(utoipa_axum::routes!(analysis::period::get_period))
        .routes(utoipa_axum::routes!(
            analysis::activity::overview::get_overview
        ))
        .routes(utoipa_axum::routes!(
            analysis::activity::timeline::get_timeline
        ))
        .routes(utoipa_axum::routes!(
            analysis::lifelog::overview::get_lifelog_overview
        ))
        .routes(utoipa_axum::routes!(
            analysis::lifelog::timeline::get_lifelog_timeline
        ))
        .routes(utoipa_axum::routes!(
            routines::list::get_routines,
            routines::save::post_routine
        ))
        .routes(utoipa_axum::routes!(routines::suggestions::get_suggestions))
        .routes(utoipa_axum::routes!(
            routines::save::put_routine,
            routines::delete::delete_routine
        ))
        .routes(utoipa_axum::routes!(routines::occurrences::get_occurrences))
        .layer(Extension(factory))
        .layer(Extension(snapshot))
}
