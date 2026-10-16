SHELL := /bin/bash
.DEFAULT_GOAL := help

# LIVE settings come from .env, STAGE settings from .env.stage; with-env.sh applies
# one of them to a command (never both). .env is parsed by the scripts, not by Make,
# so `$` and `#` in it are safe and it cannot override the shell's environment.
LIVE_ENV ?= .env
LIVE := scripts/with-env.sh $(LIVE_ENV)
STAGE := scripts/with-env.sh .env.stage
# Where `make start` serves. Priority: `make start START_ADDR=...`, then the
# START_ADDR environment variable, then START_ADDR in .env, then the default.
ifeq ($(origin START_ADDR),undefined)
START_ADDR := $(shell . scripts/lib/db-url.sh; a=$$(env_value "$(LIVE_ENV)" START_ADDR); echo "$${a:-127.0.0.1:7400}")
endif

.PHONY: help setup start dev stage web-build db db-stop stage-db stage-db-stop \
	test smoke check gen-api backup restore backup-stage restore-stage

help: ## List targets (addresses and ports are in .env and .env.stage)
	@echo "LIVE (your real data, .env)"
	@grep -hE '^[a-z-]+:.*## \[live\]' Makefile | sed -E 's/:.*## \[live\] */|/' | awk -F'|' '{printf "  make %-14s %s\n", $$1, $$2}'
	@echo "STAGE (throwaway data, .env.stage: tests and checks)"
	@grep -hE '^[a-z-]+:.*## \[stage\]' Makefile | sed -E 's/:.*## \[stage\] */|/' | awk -F'|' '{printf "  make %-14s %s\n", $$1, $$2}'
	@echo "OTHER"
	@grep -hE '^[a-z-]+:.*## \[other\]' Makefile | sed -E 's/:.*## \[other\] */|/' | awk -F'|' '{printf "  make %-14s %s\n", $$1, $$2}'

setup: ## [other] One-time: create .env files and install web deps
	@[ -f .env ] || cp .env.example .env
	@[ -f .env.stage ] || cp .env.stage.example .env.stage
	@[ -f web/.env ] || cp web/.env.example web/.env
	@[ -d web/node_modules ] || (cd web && pnpm install)

# ---- LIVE ------------------------------------------------------------------

start: setup db backup web-build ## [live] Use the app: build it, serve it and the API at START_ADDR (see .env)
	@echo "Vitarchy: http://$(START_ADDR)  (Ctrl+C stops)"
	$(LIVE) env WEB_DIST=web/dist BIND_ADDR=$(START_ADDR) cargo run --release -p vitarchy-server

dev: setup db backup ## [live] Hot-reload dev on LIVE data (ports: .env and web/.env)
	@trap 'kill $$(jobs -p) 2>/dev/null' EXIT INT TERM; \
	$(LIVE) cargo run -p vitarchy-server & \
	$(LIVE) pnpm --dir web dev & \
	wait

web-build: setup ## [live] Build the web app into web/dist
	VITE_API_BASE_URL= pnpm --dir web build

db: ## [live] Start the live Postgres and wait until healthy
	scripts/compose.sh live up -d --wait postgres

db-stop: ## [live] Stop the live Postgres (data is kept)
	scripts/compose.sh live stop postgres

backup: db ## [live] Dump live to backups/vitarchy-<time>.sql.gz (keeps the last 10)
	scripts/backup.sh live

restore: db ## [live] Replace live data with a backup, after a y/N prompt: make restore FILE=backups/...
	@[ -n "$(FILE)" ] || { echo "usage: make restore FILE=backups/vitarchy-<time>.sql.gz"; exit 2; }
	scripts/restore.sh live "$(FILE)"

# ---- STAGE -----------------------------------------------------------------

stage: setup stage-db ## [stage] Hot-reload dev on STAGE data (ports: .env.stage)
	@trap 'kill $$(jobs -p) 2>/dev/null' EXIT INT TERM; \
	$(STAGE) cargo run -p vitarchy-server & \
	$(STAGE) pnpm --dir web dev & \
	wait

stage-db: setup ## [stage] Start the throwaway stage Postgres (RAM only)
	scripts/compose.sh stage up -d --wait postgres-stage

stage-db-stop: ## [stage] Stop the stage Postgres (its data is discarded)
	scripts/compose.sh stage rm -sf postgres-stage

test: stage-db ## [stage] Run Rust and web tests against stage
	$(STAGE) cargo test --workspace
	pnpm --dir web test

smoke: stage-db ## [stage] Start a stage server, check health and a settings save, stop it
	scripts/smoke.sh

backup-stage: stage-db ## [stage] Back up the stage database (for testing the backup scripts)
	scripts/backup.sh stage

restore-stage: stage-db ## [stage] Restore a backup into stage, no prompt: make restore-stage FILE=backups/stage/...
	@[ -n "$(FILE)" ] || { echo "usage: make restore-stage FILE=backups/stage/vitarchy-<time>.sql.gz"; exit 2; }
	scripts/restore.sh stage "$(FILE)"

check: stage-db ## [stage] Lint, typecheck and test everything (the Definition of done gate)
	scripts/test-guards.sh
	cargo clippy --workspace --all-targets -- -D warnings
	$(STAGE) cargo test --workspace
	cd web && pnpm typecheck && pnpm lint && pnpm test

# ---- OTHER -----------------------------------------------------------------

gen-api: ## [other] Regenerate the TS API client from the Rust OpenAPI spec
	cd web && pnpm gen:api
