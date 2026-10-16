# Developing Vitarchy

Everything you need to change Vitarchy. For running it, see the [README](../README.md). Terms are defined in [GLOSSARY.md](../GLOSSARY.md), and the reasons behind the big decisions are in [docs/adr/](adr/).

## Architecture

```
browser ──HTTP──> Rust server (Axum) ──> Postgres (Docker)
   └─ React SPA: served by the Rust server (make start), or by Vite proxying /api (make dev)
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

## Environments

Two environments with separate Postgres instances (ADR 0007). **Live** holds the person's real data. **Stage** is throwaway; everything automated runs there.

| | Live | Stage |
|---|---|---|
| Postgres | service `postgres`, `127.0.0.1:5433`, db `vitarchy`, named volume `vitarchy-pgdata` | service `postgres-stage`, `127.0.0.1:5434`, db `vitarchy_stage`, tmpfs (gone when stopped) |
| Env file | `.env` | `.env.stage` (gitignored; `make setup` copies `.env.stage.example`) |
| Run the app | `make start`: server serves `web/dist` and the API at `START_ADDR` | `make stage`: API `127.0.0.1:3100`, Vite `127.0.0.1:7520` |
| Hot-reload dev | `make dev`: API `127.0.0.1:3000`, Vite `127.0.0.1:7420`, on live data | `make stage` |
| Tests and checks | never | `make test`, `make check`, `make smoke` |
| Backup | `make backup` (also run by `start` and `dev`), `make restore FILE=...` | `make backup-stage`, `make restore-stage FILE=...` |

- **Tests:**
  - Rust integration tests use `vitarchy_core::test_db::TestDb` (feature `test-support`), which creates a uniquely named database on stage, drops it when the test ends, and sweeps leftovers older than an hour from crashed runs.
  - The only connection setting is `TEST_DATABASE_URL`, which `scripts/with-env.sh` sets from `.env.stage`'s `DATABASE_URL`. There is no fallback to `DATABASE_URL`, and a URL that points at live panics before connecting.
  - **What counts as live** is decided in one way everywhere: compare against the live `DATABASE_URL` (`.env`, else `.env.example`): same server (host and port, where `localhost`, `127.0.0.1` and `::1` are equal) or the same database name. Shell: `scripts/lib/db-url.sh`. Rust: `test_db::check_against`, with the live URL taken from `LIVE_DATABASE_URL` (exported by `with-env.sh`), else `.env` at the workspace root, else the shipped defaults (port 5433, database `vitarchy`). `scripts/test-guards.sh` tests the shell side and runs in `make check`.
  - Running `cargo test` directly needs `TEST_DATABASE_URL` and `make stage-db`.
- **Env precedence (`scripts/with-env.sh FILE cmd`):** the app's variables are cleared from the caller's shell first, then `FILE` is applied, then anything the command sets itself (`with-env.sh .env.stage env WEB_DIST=x ...`) wins. Stage runs also set `VITARCHY_STAGE=1`, which makes the server skip `.env`, and refuse a stage file whose `DATABASE_URL` is live, however the path is written.
- **Agent and manual checks:** use `make smoke` or `scripts/smoke.sh` (both run on stage: starts a stage server, checks health and a settings save, stops it; it refuses a live URL) or `make stage`. To try the production build, run `scripts/with-env.sh .env.stage env WEB_DIST=web/dist BIND_ADDR=127.0.0.1:7401 target/release/vitarchy-server` (variables after `env` win over the file).
- **Serving the app:** with `WEB_DIST` set, the server also serves that directory, falling back to `index.html` only for navigation-like paths (no file extension, not under `/assets/`); a missing file or asset is a 404. `/api/*` keeps priority and an unknown `/api` path is a 404 problem JSON. Unset (as in dev), the server is API-only. `make web-build` forces `VITE_API_BASE_URL` empty, so the built app calls the API on its own origin.
- **Ports and names:** each env file's `DATABASE_URL` is the only place a Postgres port, user or database name is written. `scripts/compose.sh live|stage ...` (used by the `db` targets and the backup scripts) derives them from it for `docker-compose.yml`, whose `${VAR:-default}` values only serve a bare `docker compose`. The Vite proxy takes the API port from `BIND_ADDR`.
- **Backups:** `scripts/backup.sh` runs `pg_dump` inside the container into `backups/` (stage: `backups/stage/`) as `vitarchy-<time>-<NN>.sql.gz` (never overwrites; the counter separates same-second backups) and keeps the newest 10. `scripts/restore.sh` copies the chosen file aside, takes a fresh backup, replaces the database in one transaction, and for live asks y/N. The file being restored is never rotated away.
- **Never** run `docker compose down -v` or touch the `vitarchy-pgdata` volume: it is the live data.

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

**Reading your settings.** Call `useModuleSettings` from the manifest file (the only `app/` file a Module may import). The same file exports `useCoreSettings()` for reading Core settings such as `week_start`:

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

## Time module

Time keeps two things apart: **Time Entries** (what you record in Toggl, brought in by **Sync**) and **Activity** (what the Toggl Track app noticed, brought in by **Import**). Open Time in the app; both run from buttons.

- **Connect Toggl:** paste your Toggl API token (Toggl profile > API token) into the Connect card. The server checks it, stores it, and never shows or logs it again. Disconnect forgets it but keeps synced entries. Connecting a different Toggl user or workspace clears the old one's entries, projects, clients and tags (they are Toggl's, and the next Sync fetches the new ones); Activity is never cleared.
- **Sync now:** brings the last 3 months of Time Entries and keeps the copy matching Toggl (a deleted entry disappears here too). The first Sync, a reconnect, or one after 80+ days away reads the whole window and also removes entries Toggl no longer lists; a later Sync reads only changes. Only one Sync runs at a time. Entries of other workspaces are ignored, and running timers wait for a later Sync (one that was synced and is now running leaves the copy until it stops). A Toggl rate limit shows when it resets.
- **Import activity:** reads the latest Snapshot (see [Activity snapshot helper](#activity-snapshot-helper)) and only adds or refreshes rows, never deletes, so Vitarchy keeps Activity that Toggl later discards. A repeat Import changes nothing. An ongoing activity gets its later end on the next Import. Rows with an unreadable id or impossible times are skipped and counted.
- **Stage and tests:** `TIME_ACTIVITY_SNAPSHOT` points the server at another Snapshot file. Tests build a synthetic one in a temporary folder.
- **Troubleshooting:**
  - "No Snapshot yet": the helper has not run. `make helper-install`, grant Full Disk Access, then `make helper-run`.
  - "The Snapshot could not be read": the file is damaged or not Toggl's database. `make helper-run` takes a fresh one.
  - Snapshot looks old: it is at most 8 hours old by design. `make helper-run` takes one now.
  - Server logs: `activity import finished` carries `read`, `inserted`, `updated`, `unchanged`, `skipped`, `duration_ms` and the request id. Titles and app names are never logged.

## Time analysis

Analysis runs on the server ([ADR 0010](adr/0010-analysis-computed-on-the-server.md)); the web draws ready-made series with Recharts, themed by `@/design` (`axisProps`, `gridProps`, `tooltipProps`, `seriesColor(i)`; series colours are the `--chart-N` tokens, validated in both themes; a ninth series folds into `OTHER_COLOR`).

- **Period and Days:** `crates/modules/time/src/analysis/period.rs` turns `from`, `to` (`YYYY-MM-DD`) and `tz` (IANA) into local Days with exact UTC bounds, at most 400. `day_split.rs` cuts any `[start, end)` span into per-Day portions. Both are pure and DST-safe. Every analysis route takes `PeriodQuery` (`routes/analysis/params.rs`), and bad input is a 400 problem with a `code`.
- **Activity analysis:** `analysis/activity/` holds one pure aggregation per file (`totals`, `heatmap`, `bounds`, `timeline`, `trend`), each splitting rows at local midnights first; Activity is counted raw (no idle correction). Routes are `GET /api/time/analysis/activity/{totals,heatmap,bounds,timeline,trend}` (`routes/analysis/activity/`).
- **Context rules:** `analysis/context_rules.rs` is the table (editor and browser apps, known sites, the editor separator); `analysis/context.rs` is the logic. To teach Vitarchy a new site or editor, add a row there and a test in `context.rs`.
- **Web:** the Time page keeps its tab and Period in the URL (`?tab=activity&period=week&from=2026-10-12`). `modules/time/period.ts` and `timeSearch.ts` are the pure logic. Tab bodies load lazily (`tabBodies.ts`), so Recharts is only fetched when Activity opens. `modules/time/shared/DayTimeline.tsx` is the single-day lane the Life log tab will reuse.
- **Synthetic data for checks:** `make seed-stage` fills stage with about 60 days of invented life-log Time Entries (office commute, sleep, workout, dinner, bath). It refuses anything but stage, needs the stage server to have run once so the schema exists, and is safe to repeat. `SEED_TZ` sets the zone the days are lived in (default `Asia/Kolkata`).

## Activity snapshot helper

Toggl Track's Activity lives in a macOS-protected folder. `crates/helpers/activity-snapshot` is a small signed binary, run by launchd, that copies it into `~/Library/Application Support/Vitarchy/activity/latest.sqlite` (the Snapshot). It has its own Full Disk Access grant, so the terminal and editor need none. Why: [ADR 0009](adr/0009-activity-via-signed-snapshot-helper.md).

- **Install:**
  1. `make signing-setup` once. It creates the self-signed `Vitarchy Dev` identity in its own keychain, and refuses to run again without `scripts/setup-signing.sh --force`, because a new identity revokes the Full Disk Access grant.
  2. `make helper-install` builds, signs and verifies the binary, installs it to `~/Library/Application Support/Vitarchy/bin/activity-snapshot`, registers the launchd agent (every 8 hours and at login), and opens System Settings.
- **Grant Full Disk Access:** in Privacy & Security > Full Disk Access, click `+`, press Cmd+Shift+G, paste the binary path that `make helper-install` printed, and switch it on.
- **Verify:** `make helper-run`, then `tail -n 3 ~/Library/Logs/Vitarchy/activity-snapshot.log`. A good run logs `outcome=ok bytes=... duration_ms=...`, and `latest.sqlite` exists. Its modification time is the Snapshot time. A failure logs `outcome=error kind=...` (never database content), and `kind=sqlite` usually means the grant is missing.
- **Uninstall:** `make helper-uninstall` removes the agent and binary but keeps Snapshots. Remove the Full Disk Access entry by hand.
- **Tests and stage:** `ACTIVITY_SNAPSHOT_SOURCE` and `ACTIVITY_SNAPSHOT_DEST_DIR` override the source database and destination folder. Never run the helper by hand from a terminal to test Full Disk Access: launchd (`make helper-run`) is what macOS attributes the access to.

## Design system

The look is "Chroma + Quest" (light and dark) on shadcn/ui, Tailwind v4 and Motion. Import everything from the barrel: `import { Button, XpBar } from "@/design"`. ESLint blocks deep paths.

```
design/theme     tokens.css (colours per theme, radius, shadows, timing), base.css (grain, glow), fonts
design/ui        shadcn components (`pnpm dlx shadcn add <name>`; keep them close to upstream)
design/controls  Vitarchy controls built on ui (SegmentedControl)
design/effects   copied-in Magic UI effects (NumberTicker, BorderBeam, MagicCard)
design/motion    spring presets, MotionProvider, MotionButton, Pressable
design/quest     XpBar, LevelBadge, StreakFlame, AchievementBadge, StatCard, QuestList
design/charts    chart theme from tokens (series colours, axis/grid/tooltip props, ChartTooltip, ChartLegend)
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
make check        # clippy, cargo test, typecheck, lint, vitest (on stage)
make gen-api      # after changing any Rust API type
cd web && pnpm build
```

Rust tests need the stage Postgres, which `make test` and `make check` start. See Environments.

## Configuration

| File | Variable | Default | Purpose |
|---|---|---|---|
| `.env` | `DATABASE_URL` | `postgres://vitarchy:vitarchy@127.0.0.1:5433/vitarchy` | Postgres connection |
| `.env` | `BIND_ADDR` | `127.0.0.1:3000` | API address (loopback only) |
| `.env` | `RUST_LOG` | `info`, with `sqlx`, `hyper`, `h2` and `tower` quieted to `warn` | Server log level filter |
| `.env` | `LOG_FORMAT` | `pretty` | Server log output: `pretty` or `json` |
| `.env` | `START_ADDR` | `127.0.0.1:7400` | Where `make start` serves the app and API (`START_ADDR=... make start` overrides it) |
| env | `WEB_DIST` | unset | Directory of the built web app for the server to serve (`make start` sets `web/dist`) |
| `.env.stage` | `DATABASE_URL`, `BIND_ADDR`, `WEB_PORT` | stage values (`:5434`, `3100`, `7520`) | The stage environment, same meanings as above |
| env | `TEST_DATABASE_URL` | set by `make test`/`make check` | The only database Rust tests use; must not be live |
| `web/.env` | `WEB_PORT` | `7420` | UI dev and preview port |
| `web/.env` | `API_PORT` | `3000` | Where the dev proxy sends `/api` when `BIND_ADDR` is not set (a bare `pnpm dev`) |
| `web/.env` | `VITE_LOG_LEVEL` | `debug` in dev, `warn` in builds | Browser console log level (`debug`, `info`, `warn`, `error`, `silent`) |
| `web/.env` | `VITE_API_BASE_URL` | empty (same origin) | API base URL for non-proxied builds such as Tauri |

### Logging (server)

Logs go to stderr through `tracing`; `core::telemetry::init` picks the format (`LOG_FORMAT=pretty|json`) and the level (`RUST_LOG`).

- **Requests:** `server/src/request_log.rs` gives every request an `x-request-id` (UUID, or the caller's if sent), echoes it on the response, and logs one `response` line. A caller's id is kept only if it is 1-64 characters of `A-Za-z0-9._-`; otherwise a new UUID is used. `request_id`, `method` and the matched `path` are **span fields** (under `spans` in `LOG_FORMAT=json`, on every line logged while handling the request); `status` and `latency_ms` are **event fields** on the `response` line itself. The level comes from `core::level_for_status` (5xx `error`, 4xx `warn`, the rest `info`), the same rule `ApiError` uses; healthy `/api/health` polls are `debug`.
- **Errors:** `ApiError` logs once when rendered, at the level `level_for_status` gives. 5xx includes the source error (internal); 4xx logs only the status and a fixed kind (`bad_request`, `not_found`, `conflict`), because the message can quote what the person entered.
- **Startup:** the config (database password masked), bind address, mode (`api only` or serving `web_dist`) and registered Core features and Modules.
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
