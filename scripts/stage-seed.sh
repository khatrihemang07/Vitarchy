#!/usr/bin/env bash
# Fills STAGE with fake Time data (projects, clients, tags, ~100 Time Entries).
#   usage: scripts/stage-seed.sh        (or `make stage-seed`)
# Only ever targets the stage Postgres: db-target.sh refuses a live DATABASE_URL, and
# the SQL goes through the stage container, so it cannot reach the live one.
set -euo pipefail
# shellcheck source=scripts/db-target.sh
. "$(dirname "$0")/db-target.sh" stage
cd "$ROOT"
compose exec -T "$SERVICE" psql -q -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 \
  <scripts/stage-seed-time.sql
