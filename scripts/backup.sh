#!/usr/bin/env bash
# Dumps a database to <backup dir>/vitarchy-<YYYYmmdd-HHMMSS>-<NN>.sql.gz and keeps
# the newest 10 per environment. Read-only on the database (pg_dump only).
#   usage: scripts/backup.sh live|stage [--protect FILE]
# --protect: never rotate FILE away (restore.sh passes the backup being restored).
# With FINANCE_STATEMENTS_DIR set (in the env file, or the live default folder) and not
# empty, the Statement originals are archived next to the dump as <same name>.statements.tar.gz.
# Names end in a counter, so two backups in one second never collide and name order
# is time order.
set -euo pipefail
# shellcheck source=scripts/db-target.sh
. "$(dirname "$0")/db-target.sh"
cd "$ROOT"
target=$1
shift

protected=
if [ "${1:-}" = --protect ]; then protected=${2:?--protect needs a FILE}; fi

KEEP=10
mkdir -p "$BACKUP_DIR"

# Claim a name that does not exist yet; noclobber makes the claim atomic, so a file
# is never overwritten.
stamp=$(date +%Y%m%d-%H%M%S)
n=0
while :; do
  n=$((n + 1))
  dest=$(printf '%s/vitarchy-%s-%02d.sql.gz' "$BACKUP_DIR" "$stamp" "$n")
  ( set -o noclobber; : >"$dest" ) 2>/dev/null && break
done
tmp_path="$dest.partial"
trap 'rm -f "$tmp_path" "${dest%.sql.gz}.statements.tar.gz.partial"; [ -s "$dest" ] || rm -f "$dest"' EXIT

# --clean --if-exists makes the dump replace existing objects when restored.
compose exec -T "$SERVICE" pg_dump -U "$DB_USER" --clean --if-exists --no-owner "$DB_NAME" | gzip >"$tmp_path"
[ -s "$tmp_path" ] && [ "$(gzip -dc "$tmp_path" | wc -c)" -gt 0 ] || { echo "backup is empty; keeping nothing" >&2; exit 1; }
mv "$tmp_path" "$dest"
echo "backup written: $dest ($(du -h "$dest" | cut -f1))"

# Statement originals. Skipped quietly when there is no folder or it is empty.
statements_dir=$(statements_dir_for "$target")
if [ -n "$statements_dir" ] && [ -d "$statements_dir" ] && [ -n "$(ls -A "$statements_dir")" ]; then
  archive=${dest%.sql.gz}.statements.tar.gz
  tar -czf "$archive.partial" -C "$statements_dir" . && mv "$archive.partial" "$archive"
  echo "statements written: $archive ($(du -h "$archive" | cut -f1))"
fi

# The glob expands in name order, which is oldest first. Delete from the oldest end
# until $KEEP remain, skipping the protected file.
backups=("$BACKUP_DIR"/vitarchy-*.sql.gz)
excess=$((${#backups[@]} - KEEP))
for old in "${backups[@]}"; do
  [ "$excess" -gt 0 ] || break
  [ -n "$protected" ] && [ "$old" -ef "$protected" ] && continue
  rm -f "$old" "${old%.sql.gz}.statements.tar.gz"
  excess=$((excess - 1))
done
