use sqlx::migrate::Migrator;
use utoipa_axum::router::OpenApiRouter;

use crate::AppState;

/// Everything the server needs to know about one Module.
///
/// A Module crate exposes a single function returning this value; the server's
/// registry lists those functions. The router is mounted at `/api/{id}`.
///
/// **Schema convention.** Each Module owns exactly one Postgres schema, named
/// after its `id`. Its migrations must create and use only that schema and
/// never touch another Module's. (sqlx's own tracking table lives in the
/// default schema and is shared by all Modules; see [`Module::new`].)
///
/// Fields are private so the only way to build a Module is [`Module::new`],
/// which keeps the shared-tracking-table setup from being skipped.
pub struct Module {
    id: &'static str,
    router: fn() -> OpenApiRouter<AppState>,
    migrator: Migrator,
}

impl Module {
    /// Builds a Module. The migrator is told to ignore migrations applied by
    /// other Modules, so all Modules can share sqlx's single tracking table.
    #[must_use]
    pub fn new(
        id: &'static str,
        router: fn() -> OpenApiRouter<AppState>,
        mut migrator: Migrator,
    ) -> Self {
        migrator.set_ignore_missing(true);
        Self {
            id,
            router,
            migrator,
        }
    }

    /// The Module id: its API prefix (`/api/{id}`) and its Postgres schema name.
    #[must_use]
    pub const fn id(&self) -> &'static str {
        self.id
    }

    /// Builds this Module's routes, relative to `/api/{id}`.
    #[must_use]
    pub fn router(&self) -> OpenApiRouter<AppState> {
        (self.router)()
    }

    /// The Module's migrations (already set to ignore other Modules' entries).
    #[must_use]
    pub const fn migrator(&self) -> &Migrator {
        &self.migrator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ignores_migrations_applied_by_other_modules() {
        let module = Module::new("demo", OpenApiRouter::new, Migrator::DEFAULT);
        assert!(module.migrator().ignore_missing);
        assert_eq!(module.id(), "demo");
    }
}
