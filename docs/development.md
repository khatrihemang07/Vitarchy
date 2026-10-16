# Developing Vitarchy

Everything you need to change Vitarchy. For running it, see the [README](../README.md). Terms are defined in [GLOSSARY.md](../GLOSSARY.md), and the reasons behind the big decisions are in [docs/adr/](adr/).

## Architecture

```
browser ──HTTP──> Rust server (Axum, 127.0.0.1:3000) ──> Postgres (Docker, 127.0.0.1:5433)
   └─ React SPA (Vite dev server on 127.0.0.1:7420 proxies /api)
```

- **Server-first.** A Tauri desktop app may wrap the same SPA later and call the same server (ADR 0001).
- **The API contract is OpenAPI.** Rust handlers are annotated with utoipa, and `make gen-api` regenerates `web/src/api/schema.d.ts`. If you change a Rust type, the TypeScript type check fails until you regenerate (ADR 0004).
- **Local-only.**
  - The server refuses any non-loopback `BIND_ADDR`.
  - The app loads nothing from the internet: fonts come from `@fontsource-variable/*` and every asset is bundled.
  - Installing packages while developing is fine.

```
crates/core       Module contract, AppState, ApiError/Problem, ApiJson extractor (no domain types)
crates/settings   Core feature: settings storage and API (schema `settings`)
crates/server     composition root: registry (CORE + MODULES), health, migrations, openapi CLI
web/src/app       shell, router, home, settings page, theme, module registry
web/src/design    design system (see below)
web/src/api       generated schema + typed client and hooks
web/src/modules   one folder per Module
```

## Adding a Module

**Backend:**
1. Create the crate `crates/modules/<id>`. It may depend only on `vitarchy-core`, and it exports `fn module() -> Module` built with `Module::new`.
2. The Module owns one Postgres schema named after its id. Its migrations create and use only that schema.
3. Add the crate to the workspace `members` in the root `Cargo.toml`, to the dependencies of `crates/server/Cargo.toml`, and as one line in `MODULES` in `crates/server/src/registry.rs`. (Cargo cannot use a `crates/modules/*` glob until that folder exists.)

Use `ApiJson<T>` from core for request bodies, so bad input returns the standard 400 problem JSON.

**Frontend:**
1. Create the folder `web/src/modules/<id>/`. Its `index.ts` exports a `ModuleManifest`: `id`, `label`, `accent`, `icon`, `routes`, `navItem`, and optionally `settings`.
2. Add one entry to `web/src/app/modules.ts`.

The nav, routes, ⌘K entries, accent colour and Settings section all come from the manifest.

A Module may import only `@/design`, `@/api`, `@/lib` and `@/app/module-manifest`, and never another Module. `web/src/lib` holds shared plumbing (such as the logger): everyone may import it, and it imports nothing from `app`, `api`, `design` or Modules. ESLint enforces this.

**Module settings.** Declare your settings in the manifest and the Settings page renders them as a section:

```ts
settings: {
  label: "Finance",
  icon: Wallet,
  fields: [
    { kind: "select", key: "currency", label: "Currency", default: "GBP",
      options: [{ value: "GBP", label: "£ GBP" }, { value: "EUR", label: "€ EUR" }] },
    { kind: "toggle", key: "roundUp", label: "Round up savings", default: false },
  ],
},
```

- **Field kinds:** `select`, `toggle`, `text` and `number`. To add a new kind, extend `SettingField` in `app/settings/schema.ts` and register a renderer in `app/settings/fields/`.
- **Storage:** values are saved under the Module's id.
- **Defaults and bad values:** missing or invalid values fall back to the declared defaults.

**Reading your settings.** Call `useModuleSettings` from the manifest file (the only `app/` file a Module may import):

```ts
import { useModuleSettings } from "@/app/module-manifest";
import { settings } from "./settings"; // `export const settings = {...} as const satisfies ModuleSettings`, also used in the manifest

function Budget() {
  const { values, save } = useModuleSettings("finance", settings);
  values.currency; // "GBP" | "EUR": the stored value, or the declared default
  save("roundUp", true); // saves just this key; the Settings page shows the same value
}
```

- `values` always has every declared key, with defaults for anything missing or invalid, and the declared defaults while settings are still loading.
- `save(key, value)` applies the change at once and saves it under your Module id. Saves are queued in order, and a failed save reverts to what the server has.
- Outside React, `withDefaults(settings, stored)` (also exported from the manifest file) does the same merge.

## Settings

- **Core settings:** display name, theme, motion, week start, clock and date format. They are a typed Rust struct in `crates/settings`, and every field has a default, so adding a field needs no data migration.
- **The page:** `/settings/$section` is built from `coreSections` plus each Module's `settings`.
- **Saving:** every change saves automatically through `useSaveSettings(scope)` in `web/src/api/settings.ts`. Each change is built from the latest cached settings and shown at once; saves in a scope run in order; if one fails, the settings are refetched so the page shows what the server has, and the section shows the error until a later save succeeds.
- **Reading:** the server fills missing values with defaults, and a stored value that is no longer valid resets only that one field.
- **Theme and motion on load:** both are applied before the first paint by `web/public/appearance-boot.js`, which reads a localStorage cache. After that, `AppearanceController` keeps them in sync.

### Adding a Core setting

A new Core setting touches exactly these places:

1. `crates/settings/src/core_settings.rs`: add the field to `CoreSettings` (and an enum for it if it is a choice), and one line for it in `CoreSettings::from_stored`. Add a test in `crates/settings/tests/core_settings.rs`.
2. `make gen-api`: regenerates `web/src/api/schema.d.ts`, so TypeScript now knows the field.
3. `web/src/app/settings/core-sections.ts`: add the field to a section, with its default and options. `DEFAULT_CORE`, the Settings page and the valid theme and motion lists are all derived from this file.

Only if it is a new theme or motion value: also add it to the lists in `web/public/appearance-boot.js`. `appearance-boot.test.ts` fails until you do. Anything that should act on the new setting reads it with `useCoreSettings()`.

See ADR 0006 for why settings are stored as one JSON document per scope.

## Design system

The look is "Chroma + Quest" (light and dark) on shadcn/ui, Tailwind v4 and Motion. Import everything from the barrel: `import { Button, XpBar } from "@/design"`. ESLint blocks deep paths.

```
design/theme     tokens.css (colours per theme, radius, shadows, timing), base.css (grain, glow), fonts
design/ui        shadcn components (`pnpm dlx shadcn add <name>`; keep them close to upstream)
design/controls  Vitarchy controls built on ui (SegmentedControl)
design/effects   copied-in Magic UI effects (NumberTicker, BorderBeam, MagicCard)
design/motion    spring presets, MotionProvider, MotionButton, Pressable
design/quest     XpBar, LevelBadge, StreakFlame, AchievementBadge, StatCard, QuestList
design/brand     LogoMark (same artwork as public/favicon.svg)
```

- **Rules:**
  - All colours, radii, shadows and timings come from tokens.
  - Components in `design/` take props only and never fetch data.
  - One component per file.
  - Pure logic goes in tested functions.
- **Themes:** `tokens.css` defines both palettes, selected with `[data-theme]` on `<html>`. Add any new colour to both.
- **Module accent:** the shell sets `--module-accent` from the active route, so `text-module`, `bg-module/20`, primary buttons and focus rings all follow the active Module.
- **Showcase:** `/design` shows every primitive with sample data, in dev builds only.

## Checks

```sh
make check        # clippy, cargo test, typecheck, lint, vitest
make gen-api      # after changing any Rust API type
cd web && pnpm build
```

Rust tests need Postgres (`make db`). Database tests create and drop their own temporary database.

## Configuration

| File | Variable | Default | Purpose |
|---|---|---|---|
| `.env` | `DATABASE_URL` | `postgres://vitarchy:vitarchy@127.0.0.1:5433/vitarchy` | Postgres connection |
| `.env` | `BIND_ADDR` | `127.0.0.1:3000` | API address (loopback only) |
| `.env` | `RUST_LOG` | `info`, with `sqlx`, `hyper`, `h2` and `tower` quieted to `warn` | Server log level filter |
| `.env` | `LOG_FORMAT` | `pretty` | Server log output: `pretty` or `json` |
| `.env` | `POSTGRES_PORT` | `5433` | Host port for the Postgres container |
| `web/.env` | `WEB_PORT` | `7420` | UI dev and preview port |
| `web/.env` | `API_PORT` | `3000` | Where the dev proxy sends `/api` |
| `web/.env` | `VITE_LOG_LEVEL` | `debug` in dev, `warn` in builds | Browser console log level (`debug`, `info`, `warn`, `error`, `silent`) |
| `web/.env` | `VITE_API_BASE_URL` | empty (same origin) | API base URL for non-proxied builds such as Tauri |

### Logging (server)

Logs go to stderr through `tracing`; `core::telemetry::init` picks the format (`LOG_FORMAT=pretty|json`) and the level (`RUST_LOG`).

- **Requests:** `server/src/request_log.rs` gives every request an `x-request-id` (UUID, or the caller's if sent), echoes it on the response, and logs one `response` line. A caller's id is kept only if it is 1-64 characters of `A-Za-z0-9._-`; otherwise a new UUID is used. `request_id`, `method` and the matched `path` are **span fields** (under `spans` in `LOG_FORMAT=json`, on every line logged while handling the request); `status` and `latency_ms` are **event fields** on the `response` line itself. The level comes from `core::level_for_status` (5xx `error`, 4xx `warn`, the rest `info`), the same rule `ApiError` uses; healthy `/api/health` polls are `debug`.
- **Errors:** `ApiError` logs once when rendered, at the level `level_for_status` gives. 5xx includes the source error (internal); 4xx logs only the status and a fixed kind (`bad_request`, `not_found`, `conflict`), because the message can quote what the person entered.
- **Startup:** the config (database password masked), bind address and registered Core features and Modules.
- **Settings:** a save logs the scope and the changed key names; a stored value that no longer parses logs a `warn` with the scope and field.
- **Never log secrets or setting values.** Log scopes, keys, ids and statuses.
- **New Module crates:** nothing to do. `DEFAULT_FILTER` in `core/src/telemetry.rs` is crate-agnostic (`info` for everything, noisy dependencies quieted), so a new Module's logs show up on their own.
- Use structured fields (`tracing::info!(scope, changed = ?keys, "settings saved")`), and `#[tracing::instrument(skip_all, fields(...))]` on handlers and repo functions.

### Logging (web)

Use `createLogger(scope)` from `@/lib/log` (`debug`, `info`, `warn`, `error`), never `console.*` (ESLint `no-console` allows it only in `lib/log.ts`). Output is `[scope] message {fields}` in the browser console only; nothing is sent anywhere.

- **Level:** `VITE_LOG_LEVEL` (`debug|info|warn|error|silent`); default `debug` in `pnpm dev`, `warn` in a production build.
- **Never log secrets or setting values.** Log scopes, keys, statuses and paths.
- **API calls:** `api/requestLog.ts` logs every failed response once (`warn` for 4xx, `error` for 5xx, `error` for network failures) with method, path, status and the server's `x-request-id`. Search the server logs for that id to find the matching line.
- **Log at boundaries** (a failed save, an unknown route, an applied appearance), not in render paths or loops.
