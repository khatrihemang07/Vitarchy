# Crate per Module, a shared Core, no Module-to-Module dependencies

Each Module is its own Rust crate, and all Modules depend on one shared `vitarchy-core` crate and on nothing else in the workspace. A Module never depends on another Module, so Cargo itself enforces the boundary. Each Module owns a Postgres schema named after it, so table boundaries match crate boundaries. The frontend mirrors this with one folder per Module, with a lint rule forbidding imports between them.

Core holds no domain types until a second Module genuinely needs to share one. If Modules need to talk to each other, that goes through Core or the server, not a direct dependency. Adding a Module is one new crate plus one registry line on the backend, and one folder plus one registry entry on the frontend.
