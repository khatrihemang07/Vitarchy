SHELL := /bin/bash
.DEFAULT_GOAL := help

.PHONY: help dev db db-stop setup gen-api check

help: ## List targets
	@grep -E '^[a-z-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{printf "  make %-9s %s\n", $$1, $$2}'

setup: ## One-time: create .env files and install web deps
	@[ -f .env ] || cp .env.example .env
	@[ -f web/.env ] || cp web/.env.example web/.env
	@[ -d web/node_modules ] || (cd web && pnpm install)

db: ## Start Postgres and wait until healthy
	docker compose up -d --wait

db-stop: ## Stop Postgres
	docker compose stop

dev: setup db ## Run API (127.0.0.1:3000) and UI (127.0.0.1:5173); Ctrl+C stops both
	@trap 'kill $$(jobs -p) 2>/dev/null' EXIT INT TERM; \
	cargo run -p vitarchy-server & \
	pnpm --dir web dev & \
	wait

gen-api: ## Regenerate the TS API client from the Rust OpenAPI spec
	cd web && pnpm gen:api

check: db ## Lint, typecheck and test everything
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace
	cd web && pnpm typecheck && pnpm lint && pnpm test
