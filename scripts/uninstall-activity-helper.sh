#!/usr/bin/env bash
# Removes the activity-snapshot launchd agent and binary. Keeps existing Snapshots,
# the log, and the signing identity.
#   usage: scripts/uninstall-activity-helper.sh      (make helper-uninstall)
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
# shellcheck source=lib/helper.sh
. "$here/lib/helper.sh"

launchctl bootout "gui/$(id -u)/$HELPER_ID" 2>/dev/null || true
rm -f "$HELPER_PLIST" "$HELPER_BIN" "$HELPER_BIN.new"
echo "Removed the launchd agent and $HELPER_BIN."
echo "Kept: Snapshots, the log, and the '$SIGN_IDENTITY' identity."
echo "To clear the Full Disk Access entry: System Settings > Privacy & Security > Full Disk Access, select $HELPER_NAME and click \"-\"."
