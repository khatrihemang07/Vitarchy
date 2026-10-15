# OpenAPI as the Rust-to-TypeScript contract

The server describes its HTTP API with utoipa and serves one merged OpenAPI document (also printable with `vitarchy-server openapi`, without a database). The frontend generates its types and typed client from that document (openapi-typescript, openapi-fetch) and uses TanStack Query on top. We chose this over hand-written types or GraphQL so the Rust handlers stay the single source of truth, and a Tauri shell can reuse the same REST API unchanged.
