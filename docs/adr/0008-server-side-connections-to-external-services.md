# Server-side connections to external services

Vitarchy is local-only, but a Module may let the person connect an outside service (the first is Toggl Track). Only the Rust server talks to such a service, and only after the person connects it. The browser still loads nothing external and makes no remote calls, so the frontend rule in ADR 0001 stands unchanged.

The person's token for a service is stored server-side in that Module's schema. It is write-only: the API accepts it but never returns it, and it is never logged. We chose server-side calls over browser-side calls so a token never reaches the browser or its console and network logs, and so a Tauri shell (ADR 0001) needs no change.

## Consequences

- Outbound calls happen only when the person acts (a button), never in the background.
- A service's data is mirrored into the Module's own tables, so Vitarchy keeps working when the service is unreachable.
- Tokens are held in Postgres as plain text. That is acceptable for a single-user app on loopback, and the backups in `backups/` hold them too, so treat those files as secret.
