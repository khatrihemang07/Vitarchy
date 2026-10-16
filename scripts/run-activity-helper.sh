#!/usr/bin/env bash
# Runs the installed activity helper now, through launchd so Full Disk Access applies.
#   usage: scripts/run-activity-helper.sh        (make helper-run)
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
# shellcheck source=lib/helper.sh
. "$here/lib/helper.sh"

launchctl kickstart -k "gui/$(id -u)/$HELPER_ID"
