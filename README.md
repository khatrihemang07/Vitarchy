# Vitarchy

A personal life-management app built from Modules (Time, Finance, ...). Rust (Axum) server, Postgres, and a React SPA. See `GLOSSARY.md` and `docs/adr/`.

**Local-only.** Everything runs on your machine. The server binds to `127.0.0.1` only (a non-loopback `BIND_ADDR` is rejected), and the frontend loads no external runtime resources: no CDN scripts or fonts, no analytics, no remote API calls. All assets are bundled.

## Prerequisites
Rust (stable), Node 22+, pnpm, Docker.

## Run
```sh
make dev                          # does everything below: .env, deps, Postgres, API + UI; Ctrl+C stops both
make help                         # other targets: db, db-stop, gen-api, check
```
Or step by step:
```sh
cp .env.example .env              # once
docker compose up -d              # Postgres 17 on 127.0.0.1:5433
cargo run -p vitarchy-server      # API on http://127.0.0.1:3000
cd web && cp .env.example .env    # once (optional)
pnpm install && pnpm dev          # UI on http://127.0.0.1:5173 (proxies /api)
```
Check it: `curl localhost:3000/api/health` returns `{"status":"ok","db":"ok"}`. The OpenAPI document is at `/api/openapi.json`.

## Test and check
```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace            # needs Postgres up and DATABASE_URL (from .env)
cd web && pnpm gen:api            # regenerate src/api/schema.d.ts after API changes
pnpm typecheck && pnpm lint && pnpm test && pnpm build
```

## Adding a Module
- Backend: new crate in `crates/modules/<id>` depending only on `vitarchy-core`, exporting `fn module() -> Module` (built with `Module::new`); the Module owns one Postgres schema named after its id, and its migrations create and use only that schema; add it to the workspace members, to `crates/server/Cargo.toml`, and one line in `crates/server/src/registry.rs`.
- Frontend: new folder `web/src/modules/<id>/` whose `index.ts` exports a `ModuleManifest` (`id`, `label`, `accent`, `icon`, `routes`, `navItem`); add one entry in `web/src/app/modules.ts`. Nav, routes, command bar and accent colour all flow from the manifest. A Module imports only `@/design`, `@/api` and `app/` public helpers, never another Module.

## Design system
Dark-only "Chroma + Quest" look on shadcn/ui, Tailwind v4 and Motion. All of it lives in `web/src/design/` and is imported from the barrel (`import { Button, XpBar } from "@/design"`); deep paths are blocked by ESLint outside `design/`.
```
design/theme    tokens.css (@theme: colours, radius, shadows, timing), base.css (grain, glow), fonts.ts
design/ui       shadcn components (generated: `pnpm dlx shadcn add <name>`, close to upstream)
design/effects  copied-in Magic UI effects (NumberTicker, BorderBeam, MagicCard)
design/motion   spring presets, MotionProvider (honours reduced motion), MotionButton, Pressable
design/quest    Vitarchy primitives: XpBar, LevelBadge, StreakFlame, AchievementBadge, StatCard, QuestList
```
Everything is bundled: fonts come from `@fontsource-variable/*`, the grain texture is an inline SVG. Run `pnpm dev` and open `/design` for a showcase of every primitive (dev builds only; it uses sample data).

**Module accent.** A manifest's `accent` (any CSS colour) is all a Module needs. The shell sets `--module-accent` on its root from the active route, so `text-module`, `bg-module/20`, primary buttons and focus rings pick it up. Home uses the brand colour.

## Layout
```
crates/core     Module contract, AppState, ApiError, telemetry (no domain types)
crates/server   composition root: registry, app, health, migrations, openapi CLI
web/src/app     shell, router, home, module registry
web/src/design  design system (see above)
web/src/api     generated schema + typed client
web/src/modules one folder per Module
```
