#!/usr/bin/env bash
# docker compose for one environment, with ports and names taken from its env file.
#   usage: scripts/compose.sh live|stage <docker compose arguments...>
set -euo pipefail
# shellcheck source=scripts/db-target.sh
. "$(dirname "$0")/db-target.sh"
cd "$ROOT"
shift
compose "$@"
