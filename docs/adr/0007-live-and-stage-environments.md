# Live and stage environments

Vitarchy runs in two environments, each with its own Postgres.

- **Live** is the person's real data: the `postgres` service (default `127.0.0.1:5433`) with a named volume, configured by `.env`.
- **Stage** is throwaway: the `postgres-stage` service (default `127.0.0.1:5434`), data in RAM (tmpfs), configured by `.env.stage`. Every automated test, `make check`, `make smoke` and every agent or manual check runs here.

We chose two Postgres instances over one database with two names so a wrong URL cannot reach real data by sharing a server. Rust integration tests connect only through `TEST_DATABASE_URL` and refuse to run if it matches the live `DATABASE_URL` (same server or database name; ports and names are defined only in the env files).

**Dev runs on live data.** `make dev` hot-reloads against live. This is the person's explicit choice: they want to work on the app with their real data in view. The safety net is automatic: `make dev` and `make start` first take a `pg_dump` into `backups/` (last 10 kept), and `make restore` brings one back after a y/N confirmation. The trade-off is that a buggy migration or handler written during development can damage real data; the backup limits the loss to the work since the last start.

**Live is served by the Rust server.** `make start` builds the web app once and the Rust server serves `web/dist` and the API on one port (`127.0.0.1:7400`). There is no Node process in everyday use, and it matches the future Tauri shell (ADR 0001), where one local server backs one UI.
