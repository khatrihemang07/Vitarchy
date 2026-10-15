# Postgres for a single-user app

The app has one user, which would normally point to SQLite. We chose PostgreSQL (via sqlx, run with Docker Compose) because the user wants a proper central server database that Modules can use with schemas, richer types and migrations tooling.

The trade-off is that Postgres must be running for the app to work; SQLite would have needed no extra process. A Postgres container is therefore part of the dev setup, and the server reports a degraded health status when it cannot reach it.
