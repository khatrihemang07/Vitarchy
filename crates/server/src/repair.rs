//! One-off repair commands run from the CLI. They only wire the database to the Module that
//! owns the logic; the rules live in the Module's crate.

use sqlx::postgres::PgPoolOptions;
use vitarchy_finance::{MergeReport, merge_duplicate_accounts};

use crate::config::Config;

/// Merges Finance Accounts that share issuer, kind and visible digits (see
/// [`merge_duplicate_accounts`]). Logs ids and counts only.
///
/// # Errors
/// The environment is incomplete, the database is unreachable, or the merge failed; nothing
/// is changed then.
pub async fn finance_merge_duplicates(
    config: &Config,
) -> Result<MergeReport, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&config.database_url)
        .await?;
    Ok(merge_duplicate_accounts(&pool).await?)
}
