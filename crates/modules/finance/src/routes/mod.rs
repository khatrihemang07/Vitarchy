//! `/api/finance`. The statement-reader and the file store reach handlers as axum
//! `Extension`s (core's `AppState` stays domain-free); [`router`] layers the real ones and
//! tests call [`router_with`] to layer fakes.

mod accounts;
mod confirm;
mod overview;
mod passwords;
mod printings;
mod read_again;
mod read_again_all;
mod statements;
mod status;
mod transaction_items;
mod transactions;
mod transfer_candidates;
mod transfer_info;
mod transfers;
mod uploads;

use std::sync::Arc;

use axum::{Extension, extract::DefaultBodyLimit};
use utoipa_axum::{router::OpenApiRouter, routes};
use vitarchy_core::AppState;

use crate::{
    files::FileStore,
    reader::{ProcessReader, StatementReader},
};

/// The most a single upload request may hold (several Statements at once).
const UPLOAD_LIMIT_BYTES: usize = 50 * 1024 * 1024;

/// The production routes: the real helper and `FINANCE_STATEMENTS_DIR`.
pub fn router() -> OpenApiRouter<AppState> {
    router_with(Arc::new(ProcessReader::from_env()), FileStore::from_env())
}

/// The routes with a chosen reader and file store (tests pass fakes).
pub fn router_with(reader: Arc<dyn StatementReader>, files: FileStore) -> OpenApiRouter<AppState> {
    // Only the uploads route may take a large body.
    let uploads = OpenApiRouter::new()
        .routes(routes!(uploads::post_statements))
        .layer(DefaultBodyLimit::max(UPLOAD_LIMIT_BYTES));
    OpenApiRouter::new()
        .merge(uploads)
        .routes(routes!(statements::list_statements))
        .routes(routes!(
            statements::get_statement,
            statements::delete_statement
        ))
        .routes(routes!(confirm::post_confirm))
        .routes(routes!(read_again_all::post_read_all_again))
        .routes(routes!(read_again_all::post_read_all_again_confirm))
        .routes(routes!(read_again::post_read_again))
        .routes(routes!(read_again::post_read_again_confirm))
        .routes(routes!(accounts::list_accounts))
        .routes(routes!(accounts::patch_account))
        .routes(routes!(transactions::list_transactions))
        .routes(routes!(overview::get_overview))
        .routes(routes!(printings::list_printings))
        .routes(routes!(transfer_candidates::list_transfer_candidates))
        .routes(routes!(transfers::post_transfer))
        .routes(routes!(transfers::delete_transfer))
        .routes(routes!(passwords::list_passwords, passwords::post_password))
        .routes(routes!(passwords::delete_password))
        .routes(routes!(status::get_finance_status))
        .layer(Extension(reader))
        .layer(Extension(files))
}
