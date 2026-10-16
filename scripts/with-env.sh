#!/usr/bin/env bash
# Runs a command with every variable from an env file exported.
#   usage: scripts/with-env.sh ENV_FILE <command> [args...]
#
# Precedence (lowest to highest):
#   1. nothing from the caller's shell: the app's own variables are cleared first, so
#      a stray WEB_DIST or START_ADDR can never leak into the run;
#   2. ENV_FILE;
#   3. variables the command sets itself, e.g. `with-env.sh .env.stage env WEB_DIST=web/dist ...`.
#
# For any file other than the live `.env` ("stage"), this also:
#   - refuses to run if its DATABASE_URL is the live database (see lib/db-url.sh);
#   - exports TEST_DATABASE_URL (= DATABASE_URL, for Rust tests) and LIVE_DATABASE_URL
#     (so tests can refuse it), and VITARCHY_STAGE=1 (the server then ignores `.env`).
set -euo pipefail
# shellcheck source=scripts/lib/db-url.sh
. "$(dirname "$0")/lib/db-url.sh"
cd "$ROOT"

file=${1:?usage: with-env.sh ENV_FILE command...}
shift
case $file in /*) path=$file ;; *) path=./$file ;; esac
[ -f "$path" ] || { echo "$file is missing; run 'make setup' first" >&2; exit 1; }

live_url=$(live_database_url)
# Every variable the app, the web build or compose read from an env file.
unset DATABASE_URL BIND_ADDR START_ADDR WEB_DIST RUST_LOG LOG_FORMAT TEST_DATABASE_URL \
  LIVE_DATABASE_URL VITARCHY_STAGE POSTGRES_PORT POSTGRES_STAGE_PORT POSTGRES_DB \
  POSTGRES_STAGE_DB POSTGRES_USER POSTGRES_PASSWORD WEB_PORT API_PORT VITE_API_BASE_URL

set -a
# shellcheck disable=SC1090
. "$path"
set +a

if [ ! "$path" -ef ./.env ]; then
  if is_live_url "${DATABASE_URL:-}"; then
    echo "refusing: $file points DATABASE_URL at the LIVE database" >&2
    exit 1
  fi
  export TEST_DATABASE_URL=$DATABASE_URL LIVE_DATABASE_URL=$live_url VITARCHY_STAGE=1
fi

exec "$@"
