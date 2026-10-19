//! Applies every Module's migrations at startup.

use sqlx::PgPool;
use vitarchy_core::Module;

/// Runs each Module's migrator in registry order.
///
/// # Errors
/// Returns the first migration failure, naming nothing but the sqlx error.
pub async fn run(pool: &PgPool, modules: &[Module]) -> Result<(), sqlx::migrate::MigrateError> {
    for module in modules {
        tracing::info!(module = module.id(), "running migrations");
        module.migrator().run(pool).await?;
    }
    Ok(())
}

/// Runs each Module's startup job, in registry order, once migrations are done.
pub async fn start(pool: &PgPool, modules: &[Module]) {
    for module in modules {
        if let Some(task) = module.startup() {
            tracing::info!(module = module.id(), "running startup job");
            task(pool.clone()).await;
        }
    }
}
