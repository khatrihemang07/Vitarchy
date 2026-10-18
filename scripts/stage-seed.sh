#!/usr/bin/env bash
# Fills STAGE with fake Time data: projects, clients, tags and ~100 Time Entries, then about
# 5 weeks of Toggl and Clockify Activity, and rebuilds the Timeline from it.
#   usage: scripts/stage-seed.sh        (or `make stage-seed`)
# Only ever targets the stage Postgres: db-target.sh refuses a live DATABASE_URL, and
# the SQL goes through the stage container, so it cannot reach the live one. The Timeline
# rebuild goes to the stage server (BIND_ADDR of .env.stage) when it is running.
set -euo pipefail
# shellcheck source=scripts/db-target.sh
. "$(dirname "$0")/db-target.sh" stage
cd "$ROOT"
for sql in scripts/stage-seed-time.sql scripts/stage-seed-activity.sql; do
  compose exec -T "$SERVICE" psql -q -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 <"$sql"
done

# The Timeline is derived from Activity, so ask the stage server to build it.
bind=$(env_value "$ROOT/.env.stage" BIND_ADDR)
bind=${bind:-127.0.0.1:3100}
if [ "$bind" = "$(env_value "$ROOT/.env" BIND_ADDR)" ]; then
  echo "refusing to rebuild: .env.stage BIND_ADDR is the LIVE server's address" >&2
  exit 1
fi
if curl -fsS --max-time 5 "http://$bind/api/health" >/dev/null 2>&1; then
  curl -fsS --max-time 120 -X POST "http://$bind/api/time/timeline/rebuild"
  echo
else
  echo "stage server not running at $bind: start it (make stage) and press Rebuild Timeline on the Sources tab,"
  echo "or run: curl -X POST http://$bind/api/time/timeline/rebuild"
fi
