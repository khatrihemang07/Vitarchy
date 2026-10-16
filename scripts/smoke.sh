#!/usr/bin/env bash
# Starts a server against STAGE, checks health and a settings round-trip, stops it.
#   usage: scripts/smoke.sh        (or `make smoke`)
# It writes a setting, so it only ever runs on stage: run directly, it re-runs
# itself through with-env.sh .env.stage, and it refuses a live DATABASE_URL.
set -euo pipefail
# shellcheck source=scripts/lib/db-url.sh
. "$(dirname "$0")/lib/db-url.sh"
cd "$ROOT"
[ -n "${VITARCHY_STAGE:-}" ] || exec scripts/with-env.sh .env.stage "$0" "$@"
if is_live_url "${DATABASE_URL:-}"; then
  echo "refusing: smoke writes data and DATABASE_URL is the LIVE database" >&2
  exit 1
fi

PORT=${SMOKE_PORT:-3190}
BASE="http://127.0.0.1:$PORT"
log=$(mktemp)

cargo build -q -p vitarchy-server
BIND_ADDR="127.0.0.1:$PORT" WEB_DIST= target/debug/vitarchy-server >"$log" 2>&1 &
pid=$!
trap 'kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; rm -f "$log"' EXIT

for _ in $(seq 1 60); do
  curl -fsS "$BASE/api/health" >/dev/null 2>&1 && break
  kill -0 "$pid" 2>/dev/null || { echo "server exited early:" >&2; cat "$log" >&2; exit 1; }
  sleep 0.5
done

health=$(curl -fsS "$BASE/api/health")
echo "health:   $health"
[ "$health" = '{"status":"ok","db":"ok"}' ] || { echo "unexpected health" >&2; exit 1; }

name="smoke-$$"
curl -fsS -X PUT "$BASE/api/settings/core" -H 'content-type: application/json' -d "{\"display_name\":\"$name\"}" >/dev/null
curl -fsS "$BASE/api/settings" | grep -q "\"display_name\":\"$name\"" || { echo "settings round-trip failed" >&2; exit 1; }
echo "settings: saved and read back display_name=$name"

[ "$(curl -s -o /dev/null -w '%{http_code}' "$BASE/api/nope")" = 404 ] || { echo "unknown /api path did not 404" >&2; exit 1; }
echo "smoke ok (stage, port $PORT)"
