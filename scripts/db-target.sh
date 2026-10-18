# Sourced by backup.sh, restore.sh and compose.sh: `. scripts/db-target.sh live|stage`.
# Derives everything about the target from its env file's DATABASE_URL (the single
# source of truth): SERVICE, DB_USER, DB_PASSWORD, DB_PORT, DB_NAME, BACKUP_DIR, and
# the `compose` function that hands those values to docker-compose.yml.
# shellcheck source=scripts/lib/db-url.sh
. "$(dirname "${BASH_SOURCE[0]}")/lib/db-url.sh"

case "${1:-}" in
  live)  SERVICE=postgres;       ENV_FILE=.env;       BACKUP_DIR=$ROOT/backups ;;
  stage) SERVICE=postgres-stage; ENV_FILE=.env.stage; BACKUP_DIR=$ROOT/backups/stage ;;
  *) echo "usage: $(basename "$0") live|stage ..." >&2; exit 2 ;;
esac

_url=$(env_value "$ROOT/$ENV_FILE" DATABASE_URL)
[ -n "$_url" ] || _url=$(env_value "$ROOT/$ENV_FILE.example" DATABASE_URL)
[ -n "$_url" ] || { echo "no DATABASE_URL in $ENV_FILE (run 'make setup')" >&2; exit 1; }
if [ "$1" = stage ] && is_live_url "$_url"; then
  echo "refusing: $ENV_FILE points DATABASE_URL at the LIVE database" >&2
  exit 1
fi
IFS=$'\t' read -r DB_USER DB_PASSWORD _ DB_PORT DB_NAME < <(url_parts "$_url")

# compose ARGS...: docker compose with this target's port, user and database name.
# `--profile stage` lets compose address the profile-gated stage service.
compose() {
  if [ "$SERVICE" = postgres ]; then
    export POSTGRES_PORT=$DB_PORT POSTGRES_DB=$DB_NAME
  else
    export POSTGRES_STAGE_PORT=$DB_PORT POSTGRES_STAGE_DB=$DB_NAME
  fi
  POSTGRES_USER=$DB_USER POSTGRES_PASSWORD=$DB_PASSWORD docker compose --profile stage "$@"
}

# statements_dir_for live|stage: the folder of Statement originals (FINANCE_STATEMENTS_DIR in
# the target's env file). Live falls back to the server's default folder; stage has no
# fallback, so an unset stage never touches the real one. Prints nothing when unset.
statements_dir_for() {
  local dir
  dir=$(env_value "$ROOT/$ENV_FILE" FINANCE_STATEMENTS_DIR)
  [ -n "$dir" ] || dir=$(env_value "$ROOT/$ENV_FILE.example" FINANCE_STATEMENTS_DIR)
  if [ -z "$dir" ] && [ "$1" = live ]; then dir="$HOME/Library/Application Support/Vitarchy/statements"; fi
  printf '%s' "$dir"
}
