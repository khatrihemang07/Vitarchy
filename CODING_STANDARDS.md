# Coding standards

Every change to Vitarchy follows these rules. They exist so the codebase stays **modular, maintainable and scalable** as Modules are added one at a time. How-tos (adding a Module, a Core setting, a design primitive) live in [docs/development.md](docs/development.md); terms live in [GLOSSARY.md](GLOSSARY.md).

## Principles

- **Deep modules.** Each unit hides real work behind a small, obvious interface. Callers never need to know how it works.
- **One reason to change.** A file changes for one reason only. When a second reason shows up, split the file.
- **Single source of truth.** Every value, rule and piece of logic lives in exactly one place, and everything else derives from it (e.g. `DEFAULT_CORE` comes from the core settings sections).
- **Explainable structure.** A newcomer can say where any piece of code belongs from the folder names alone.

## Where code lives

**Backend (`crates/`)**
- `core`: shared infrastructure only (the `Module` contract, `AppState`, `ApiError`/`Problem`, `ApiJson`, telemetry). No domain types.
- `settings`: the Core feature for Settings, listed in the server's `CORE` registry.
- `server`: the composition root. It wires things together and holds no domain logic.
- `modules/<id>`: one crate per Module. It depends on `core` and nothing else, and owns the Postgres schema named after its id.

**Web (`web/src/`)**
- `app`: composes the shell, routing, pages, settings and theme.
- `design`: presentational only. It works from props, fetches nothing, and is imported through the `@/design` barrel.
- `api`: all server access, through typed clients and TanStack Query hooks.
- `lib`: cross-cutting utilities (e.g. logging) that anything may import, and that import nothing from the other folders.
- `modules/<id>`: one folder per Module. It imports only `@/design`, `@/api`, `@/lib` and `@/app/module-manifest`.

Dependencies point inward: Modules → shared layers → `core`/`lib`. ESLint and Cargo enforce these boundaries, so treat a boundary error as a design signal and resist working around it.

## Size and shape

- One component per file, and one handler group per file.
- Keep files under about 150 lines and functions under about 40. These are soft limits that tell you when to split.
- Pure logic lives in plain `.ts`/`.rs` files with tests. Components, hooks and handlers stay thin: they call that logic and render or respond.
- Express variation as data (a map or a descriptor list) rather than a growing `switch`. See `fieldRenderers` in `app/settings/fields/`.

## Naming

- Use the GLOSSARY's terms exactly. Its _Avoid_ words stay out of code, comments and UI copy.
- Names say what a thing is or does, in full words: `formatDate`, `redactUrl`, `SettingsSection`.
- Files:
  - React components are `PascalCase.tsx`.
  - Other TS files are `camelCase.ts`.
  - Rust files are `snake_case.rs`.
  - Tests sit beside their subject as `*.test.ts(x)` or under `crates/*/tests/`.

## Comments

- Comments explain **why**: intent, trade-offs, gotchas. The code already says what.
- Every exported or public item gets a one- or two-line doc comment.
- Keep comments short and current. Delete a comment the moment it stops being true.
- Track future work as a GitHub issue and reference it (`// see #12`).

## Errors

- **Rust:**
  - Handlers return `Result<_, ApiError>` and take bodies through `ApiJson<T>`, so every failure is an RFC 9457 problem response.
  - Use `?` and `thiserror`. `unwrap`/`expect` belong only in tests or on truly impossible states, with a comment saying why.
- **Web:**
  - Model outcomes as discriminated unions (see `api/health.ts`).
  - Every async UI path has a visible loading, error and empty state.

## Logging

Logs are for debugging, so make each line answer *what happened, where, and to what*.

- **Levels:**
  - `error`: something failed and needs attention.
  - `warn`: unexpected but handled.
  - `info`: a meaningful state change (startup, a save).
  - `debug`: detail for chasing a bug.
- **Structured fields, not prose:** `info!(scope, changed = ?keys, "settings saved")` and `log.warn("save failed", { scope, status })`.
- **Server:** every line carries the request's `request_id`. Responses return it as `x-request-id`, and the web client logs it, so a browser log line can be found in the server log.
- **What to log:** at boundaries (requests, saves, startup, fallbacks), not inside loops.
- **Privacy:** log identifiers and keys, never secrets or what the person entered (setting values, names, amounts).
- **Web:** use `createLogger(scope)` from `@/lib/log`. It is the only place that touches `console`.

## Visuals

- Every colour, radius, shadow, spacing and duration comes from design tokens (`design/theme/tokens.css`). Copied upstream files in `design/ui` and `design/effects` are the exception.
- Every screen works in both themes, at 390px and 1440px wide, and with reduced motion.
- Local-only: the app loads nothing from the internet at runtime, and the server binds to `127.0.0.1`.

## Tests

- Work test-first at agreed seams (the `tdd` skill): see the test go red, then make it green.
- Every pure function has unit tests. Every API route has an integration test. Every component with behaviour has a test.
- Test names describe behaviour ("rolls back when the save fails"), not implementation.
- Tests and manual checks run against throwaway data: a temporary database, or a server on its own port pointed at one. The dev database holds the person's real data and stays untouched.

## Docs

Update these in the same change as the code:
- `GLOSSARY.md` for new terms.
- An ADR in `docs/adr/` for a hard-to-reverse trade-off.
- `docs/development.md` for how-tos.
- `README.md` for anything the person using the app would notice.

## Definition of done

- [ ] `make check` passes and `cd web && pnpm build` passes.
- [ ] `make gen-api` was run if any API type changed.
- [ ] The change was viewed in a browser (ego-browser) at 390px and 1440px in both themes, if it touches UI.
- [ ] New paths log usefully, and nothing logs secrets or entered values.
- [ ] `/code-review` was run against the last commit, and its findings are fixed.
- [ ] Docs were updated as above.
