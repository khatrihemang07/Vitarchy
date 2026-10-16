# Sourced by the scripts: reads env files and decides whether a database URL is LIVE.
# The single source of truth for "is this URL live?" is the live DATABASE_URL in
# `.env` (or `.env.example` before `make setup`); nothing else hardcodes a port or name.
# The Rust twin is `vitarchy_core::test_db::check_against`; keep the rules the same.

# Repo root; VITARCHY_ROOT lets scripts/test-guards.sh use a fake one.
ROOT=${VITARCHY_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}
# Used only when neither .env nor .env.example can be read: the shipped live URL.
DEFAULT_LIVE_URL=postgres://vitarchy:vitarchy@127.0.0.1:5433/vitarchy

# env_value FILE KEY: the value of KEY in a dotenv-style file (last assignment wins,
# one pair of quotes stripped). The file is parsed, never sourced.
env_value() {
  [ -f "$1" ] || return 0
  sed -n -E "s/^[[:space:]]*$2[[:space:]]*=[[:space:]]*//p" "$1" | tail -n 1 |
    sed -E "s/[[:space:]]+\$//; s/^\"(.*)\"\$/\\1/; s/^'(.*)'\$/\\1/"
}

# live_database_url: the live DATABASE_URL (.env, then .env.example, then the default).
live_database_url() {
  local url
  url=$(env_value "$ROOT/.env" DATABASE_URL)
  [ -n "$url" ] || url=$(env_value "$ROOT/.env.example" DATABASE_URL)
  echo "${url:-$DEFAULT_LIVE_URL}"
}

# url_parts URL: prints "user<TAB>password<TAB>host<TAB>port<TAB>db", with the host
# lower-cased, localhost / 127.0.0.1 / ::1 folded to "loopback", and port 5432 if absent.
url_parts() {
  local url=$1 rest creds hostport host port db user pass
  rest=${url#*://}
  creds=
  case $rest in *@*) creds=${rest%%@*}; rest=${rest#*@} ;; esac
  hostport=${rest%%/*}
  db=${rest#*/}; db=${db%%\?*}
  case $hostport in
    \[*\]*) host=${hostport%%\]*}; host=${host#\[}; port=${hostport##*\]}; port=${port#:} ;;
    *:*) host=${hostport%%:*}; port=${hostport##*:} ;;
    *) host=$hostport; port= ;;
  esac
  host=$(printf %s "$host" | tr '[:upper:]' '[:lower:]')
  case $host in localhost | 127.0.0.1 | ::1) host=loopback ;; esac
  user=${creds%%:*}
  pass=; case $creds in *:*) pass=${creds#*:} ;; esac
  printf '%s\t%s\t%s\t%s\t%s\n' "$user" "$pass" "$host" "${port:-5432}" "$db"
}

# is_live_url URL: succeeds when URL shares a server (host and port) with the live
# URL, or names the live database anywhere.
is_live_url() {
  local _u _p host port db live_host live_port live_db
  IFS=$'\t' read -r _u _p host port db < <(url_parts "$1")
  IFS=$'\t' read -r _u _p live_host live_port live_db < <(url_parts "$(live_database_url)")
  { [ "$host" = "$live_host" ] && [ "$port" = "$live_port" ]; } || [ "$db" = "$live_db" ]
}
