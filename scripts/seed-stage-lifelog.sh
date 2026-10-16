#!/usr/bin/env bash
# Fills STAGE with ~60 days of SYNTHETIC life-log Time Entries (commute, office, sleep,
# workout, dinner, bath) so the Time analysis can be checked by eye. Never touches live.
#   usage: scripts/seed-stage-lifelog.sh     (or `make seed-stage`)
#   SEED_TZ=Europe/London scripts/seed-stage-lifelog.sh   # the zone the days are lived in
# Needs the stage Postgres up and migrated (run `make stage` once so the server migrates it).
# Safe to repeat: it replaces its own rows (ids from 9000000000) and nothing else.
set -euo pipefail
# shellcheck source=scripts/lib/db-url.sh
. "$(dirname "$0")/lib/db-url.sh"
cd "$ROOT"
[ -n "${VITARCHY_STAGE:-}" ] || exec scripts/with-env.sh .env.stage "$0" "$@"
if is_live_url "${DATABASE_URL:-}"; then
  echo "refusing: DATABASE_URL is the LIVE database" >&2
  exit 1
fi
# db-target.sh refuses a live-pointing stage file once more and sets the compose helper.
set -- stage
# shellcheck source=scripts/db-target.sh
. scripts/db-target.sh

compose exec -T "$SERVICE" psql -U "$DB_USER" -d "$DB_NAME" -q -v "tz=${SEED_TZ:-Asia/Kolkata}" -f - <scripts/seed-stage-lifelog.sql
