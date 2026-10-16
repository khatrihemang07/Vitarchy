#!/usr/bin/env bash
# Replaces a database's contents with a backup.
#   usage: scripts/restore.sh live|stage FILE
# Live asks for an explicit y/N first. A fresh backup is taken beforehand so a
# restore can itself be undone; FILE is copied aside first so that backup's
# rotation can never take away the file being restored. Stop the app first (open
# connections block it).
set -euo pipefail

# Resolve FILE before anything changes directory.
[ -n "${2:-}" ] || { echo "usage: restore.sh live|stage FILE" >&2; exit 2; }
[ -s "$2" ] || { echo "no such backup: $2" >&2; exit 1; }
source_path=$(cd "$(dirname "$2")" && pwd)/$(basename "$2")

# shellcheck source=scripts/db-target.sh
. "$(dirname "$0")/db-target.sh"
cd "$ROOT"
gzip -t "$source_path" || { echo "not a valid gzip backup: $source_path" >&2; exit 1; }

if [ "$1" = live ]; then
  printf 'This REPLACES all data in the LIVE database "%s" with %s.\nContinue? [y/N] ' "$DB_NAME" "$source_path"
  read -r answer </dev/tty || answer=
  [ "$answer" = y ] || [ "$answer" = Y ] || { echo "cancelled; nothing changed"; exit 1; }
fi

snapshot=$(mktemp)
trap 'rm -f "$snapshot"' EXIT
cp "$source_path" "$snapshot"

scripts/backup.sh "$1" --protect "$source_path"
# One transaction: a failed restore leaves the database as it was.
gzip -dc "$snapshot" | compose exec -T "$SERVICE" psql -U "$DB_USER" -d "$DB_NAME" -q -v ON_ERROR_STOP=1 --single-transaction >/dev/null
echo "restored $source_path into $1 ($DB_NAME)"
