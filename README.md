# Vitarchy

Your personal life-management app. Vitarchy grows one area of life at a time (time, money and more), each as its own **Module**, behind a colourful, game-flavoured interface.

It runs entirely on your own machine. Your data stays in a local database, and the app never talks to the internet.

## Features

- **Modular.** Each area of life is a self-contained Module. None are built yet; Time and Finance come first.
- **Settings in one place.** Theme (light, dark or system), motion, your name, week start, clock and date format. Modules add their own sections.
- **Chroma + Quest design.** Each Module has its own accent colour, with springy motion and game touches like XP, levels and streaks.
- **Private by default.** The server listens only on `127.0.0.1`, and nothing loads from the internet.

## Quick start

You need [Rust](https://rustup.rs), [Node 22+](https://nodejs.org), [pnpm](https://pnpm.io) and [Docker](https://www.docker.com).

```sh
make dev
```

Then open **http://127.0.0.1:7420**. Press Ctrl+C to stop.

The first run creates your `.env` files, installs packages and starts the database for you.

## Everyday commands

| Command | What it does |
|---|---|
| `make dev` | Start everything (database, API, app) |
| `make check` | Run every lint and test |
| `make db` / `make db-stop` | Start or stop just the database |
| `make gen-api` | Refresh the app's API types after changing the server |
| `make help` | List all commands |

## Settings

Open **Settings** from the gear at the bottom of the sidebar, or press **⌘K** and type "settings". Changes save as you make them.

## Project layout

```
crates/      Rust server: core, settings, server (and one crate per Module)
web/         React app: app shell, design system, API client, modules
docs/        development guide and architecture decisions
```

## Learn more

- [Development guide](docs/development.md): architecture, adding a Module, design system, checks
- [Glossary](GLOSSARY.md): what words like Module, Core and Setting mean here
- [Decisions](docs/adr/): why things are the way they are
