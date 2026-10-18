#!/usr/bin/env bash
# End to end over the private real Statements: a fresh `vitarchy_e2e` database on STAGE, the
# built server, the real reader, and counts-only assertions (see the driver's docstring).
#   usage: scripts/finance-e2e.sh        (or `make finance-e2e`)
# It writes data, so it only ever runs on stage: run directly, it re-runs itself through
# with-env.sh .env.stage, and it refuses a live DATABASE_URL.
set -euo pipefail
# shellcheck source=scripts/lib/db-url.sh
. "$(dirname "$0")/lib/db-url.sh"
cd "$ROOT"
[ -n "${VITARCHY_STAGE:-}" ] || exec scripts/with-env.sh .env.stage "$0" "$@"
if is_live_url "${DATABASE_URL:-}"; then
  echo "refusing: the e2e run writes data and DATABASE_URL is the LIVE database" >&2
  exit 1
fi
reader=$FINANCE_READER  # exported by with-env.sh, the one place the default is defined
[ -x "$reader" ] || { echo "the reader is not installed; run 'make finance-reader-install'" >&2; exit 1; }

cargo build -q -p vitarchy-server
exec "$(dirname "$reader")/python" crates/helpers/statement-reader/tests/golden/e2e.py
