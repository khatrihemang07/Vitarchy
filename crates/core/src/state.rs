use sqlx::PgPool;

/// State shared by every request handler.
#[derive(Clone)]
pub struct AppState {
    /// Connection pool to the one Postgres database.
    pub pool: PgPool,
}

impl AppState {
    /// Wraps the pool every handler will share.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
