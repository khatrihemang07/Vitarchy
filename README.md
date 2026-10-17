# Vitarchy

Your personal life-management app. Vitarchy grows one area of life at a time (time, money and more), each as its own **Module**, behind a colourful, game-flavoured interface.

It runs entirely on your own machine. Your data stays in a local database. The app contacts an outside service only when you connect one yourself (such as Toggl Track), and only the server does, never the browser.

## Features

- **Modular.** Each area of life is a self-contained Module. None are built yet; Time and Finance come first.
- **Activity from Toggl Track and Clockify.** The Time page can Import what each app noticed on this computer (apps, windows and, for Clockify, page addresses and idle time). Each Source has its own Import button, and a small signed helper keeps a fresh copy of each app's data (see [docs/development.md](docs/development.md#activity-snapshot-helper)).
- **Settings in one place.** Theme (light, dark or system), motion, your name, week start, clock and date format. Modules add their own sections.
- **Chroma + Quest design.** Each Module has its own accent colour, with springy motion and game touches like XP, levels and streaks.
- **Private by default.** The server listens only on `127.0.0.1`, and the browser loads nothing from the internet.

## Quick start

You need [Rust](https://rustup.rs), [Node 22+](https://nodejs.org), [pnpm](https://pnpm.io) and [Docker](https://www.docker.com).

```sh
make start
```

Then open **http://127.0.0.1:7400**. Press Ctrl+C to stop.

The first run creates your `.env` files, installs packages, starts the database, takes a backup and builds the app. Change the address with `START_ADDR` in `.env`.

## Everyday commands

| Command | What it does |
|---|---|
| `make start` | Use the app (your real data) on http://127.0.0.1:7400 |
| `make dev` | Work on the code with hot reload, on your real data: http://127.0.0.1:7420 |
| `make stage` | Hot-reload on throwaway data instead: http://127.0.0.1:7520 |
| `make check` | Run every lint and test (on throwaway data) |
| `make gen-api` | Refresh the app's API types after changing the server |
| `make help` | List all commands |

## Backups

`make start` and `make dev` back up your data first, into `backups/` (the last 10 are kept). `make backup` does it on demand.

To go back to a backup: stop the app, then `make restore FILE=backups/vitarchy-<time>.sql.gz`. It asks y/N before replacing anything.

## Settings

Open **Settings** from the gear at the bottom of the sidebar, or press **⌘K** and type "settings". Changes save as you make them.

## Project layout

```
crates/      Rust server: core, settings, server (and one crate per Module)
web/         React app: app shell, design system, API client, modules
scripts/     backup, restore and smoke-check scripts
docs/        development guide and architecture decisions
```

## Learn more

- [Development guide](docs/development.md): architecture, adding a Module, design system, checks
- [Glossary](GLOSSARY.md): what words like Module, Core and Setting mean here
- [Decisions](docs/adr/): why things are the way they are
