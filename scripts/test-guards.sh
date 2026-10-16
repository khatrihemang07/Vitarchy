#!/usr/bin/env bash
# Tests for the shell guards that keep automation off the LIVE database.
#   usage: scripts/test-guards.sh      (run by `make check`; needs no Docker)
# Each case runs against a throwaway fake repo root (VITARCHY_ROOT) so the real
# .env files are never read or changed.
set -uo pipefail
here=$(cd "$(dirname "$0")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

pass=0 fail=0
ok() { pass=$((pass + 1)); }
bad() { fail=$((fail + 1)); echo "FAIL: $1" >&2; }
# expect_status NAME WANT_STATUS command...: runs the command, compares its exit status.
expect_status() {
  local name=$1 want=$2 got
  shift 2
  "$@" >"$tmp/out" 2>&1
  got=$?
  if [ "$want" = nonzero ]; then [ "$got" -ne 0 ] && ok || bad "$name (expected failure)"
  else [ "$got" = "$want" ] && ok || { bad "$name (status $got, wanted $want)"; cat "$tmp/out" >&2; }
  fi
}
# expect_out NAME TEXT: the last command's output must contain TEXT.
expect_out() { grep -qF -- "$2" "$tmp/out" && ok || { bad "$1 (output lacks: $2)"; cat "$tmp/out" >&2; }; }

root=$tmp/root
mkdir -p "$root"
cp -R "$here/scripts" "$root/scripts"
printf 'DATABASE_URL=postgres://u:p@127.0.0.1:5500/real\nSTART_ADDR=127.0.0.1:7777\n' >"$root/.env"
export VITARCHY_ROOT=$root

# --- the live detector ------------------------------------------------------
live_check() { ( . "$here/scripts/lib/db-url.sh"; is_live_url "$1" ); }
for url in \
  postgres://u:p@127.0.0.1:5500/real \
  postgres://u:p@localhost:5500/other \
  postgres://u:p@[::1]:5500/other \
  postgres://u:p@127.0.0.1:5999/real \
  'postgres://u:p@LOCALHOST:5500/x?sslmode=disable'; do
  expect_status "is live: $url" 0 live_check "$url"
done
for url in postgres://u:p@127.0.0.1:5434/vitarchy_stage postgres://u:p@127.0.0.1:5433/vitarchy_stage; do
  expect_status "is not live: $url" 1 live_check "$url"
done

# Without any .env the conservative default (5433 / vitarchy) applies.
mv "$root/.env" "$root/.env.bak"
expect_status "default: live port" 0 live_check postgres://u:p@127.0.0.1:5433/x
expect_status "default: live name" 0 live_check postgres://u:p@127.0.0.1:9999/vitarchy
mv "$root/.env.bak" "$root/.env"

# --- with-env.sh ------------------------------------------------------------
printf 'DATABASE_URL=postgres://u:p@127.0.0.1:5434/stage\nBIND_ADDR=127.0.0.1:3100\n' >"$root/.env.stage"
WITH="$root/scripts/with-env.sh"

expect_status "with-env runs a stage file" 0 "$WITH" .env.stage true
expect_status "with-env exports TEST_DATABASE_URL and LIVE_DATABASE_URL" 0 \
  "$WITH" .env.stage sh -c 'echo "$TEST_DATABASE_URL $LIVE_DATABASE_URL"'
expect_out "with-env stage test url" "5434/stage"
expect_out "with-env live url" "5500/real"

printf 'DATABASE_URL=postgres://u:p@localhost:5500/real\n' >"$root/.env.stage"
for spelling in .env.stage ./.env.stage "$root/.env.stage"; do
  expect_status "with-env refuses live via $spelling" nonzero "$WITH" "$spelling" true
done
cp "$root/.env.stage" "$root/other.env"
expect_status "with-env refuses live via any other file" nonzero "$WITH" other.env true
expect_status "with-env allows the live file itself" 0 "$WITH" .env true

# Stage starts from a clean set of the app's variables.
printf 'DATABASE_URL=postgres://u:p@127.0.0.1:5434/stage\n' >"$root/.env.stage"
WEB_DIST=web/dist BIND_ADDR=127.0.0.1:1 START_ADDR=x TEST_DATABASE_URL=postgres://evil/live \
  expect_status "with-env cleans leaked variables" 0 \
  "$WITH" .env.stage sh -c 'echo "web=[$WEB_DIST] bind=[${BIND_ADDR-unset}] start=[${START_ADDR-unset}] test=[$TEST_DATABASE_URL]"'
expect_out "WEB_DIST cleared" "web=[]"
expect_out "BIND_ADDR dropped" "bind=[unset]"
expect_out "START_ADDR dropped" "start=[unset]"
expect_out "TEST_DATABASE_URL from stage file" "test=[postgres://u:p@127.0.0.1:5434/stage]"
# Variables after `env` still win (documented override).
expect_status "env override wins" 0 "$WITH" .env.stage env WEB_DIST=x sh -c 'echo "web=[$WEB_DIST]"'
expect_out "override applied" "web=[x]"

# --- smoke.sh is safe when run directly --------------------------------------
expect_status "smoke refuses a live DATABASE_URL" nonzero \
  env VITARCHY_STAGE=1 DATABASE_URL=postgres://u:p@127.0.0.1:5500/real "$root/scripts/smoke.sh"
expect_out "smoke says why" "LIVE"

# --- START_ADDR: env var beats .env, which beats the default -----------------
make_addr() { make -n -s -C "$here" "$@" start 2>&1 | grep -o 'BIND_ADDR=[0-9.:]*' | head -1; }
unset START_ADDR
[ "$(make_addr LIVE_ENV="$root/.env")" = "BIND_ADDR=127.0.0.1:7777" ] && ok || bad "START_ADDR from .env (got $(make_addr LIVE_ENV="$root/.env"))"
[ "$(make_addr LIVE_ENV=/nonexistent)" = "BIND_ADDR=127.0.0.1:7400" ] && ok || bad "START_ADDR default"
[ "$(START_ADDR=127.0.0.1:7500 make_addr LIVE_ENV="$root/.env")" = "BIND_ADDR=127.0.0.1:7500" ] && ok || bad "START_ADDR env override"
[ "$(make_addr LIVE_ENV="$root/.env" START_ADDR=127.0.0.1:7600)" = "BIND_ADDR=127.0.0.1:7600" ] && ok || bad "START_ADDR make argument"

echo "guards: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
