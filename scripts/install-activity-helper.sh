#!/usr/bin/env bash
# Builds, signs and installs the activity-snapshot helper as a launchd agent.
#   usage: scripts/install-activity-helper.sh        (make helper-install)
# Safe to re-run. Reinstalling under the same identity and path keeps the Full Disk
# Access grant. The helper runs every 8 hours and at login (docs/adr/0009).
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/.." && pwd)
# shellcheck source=lib/helper.sh
. "$here/lib/helper.sh"

fail() { echo "error: $*" >&2; exit 1; }

# Being listed is not being usable: a locked keychain makes codesign fail. Sign a
# throwaway file now rather than after a long compile.
preflight_signing() {
  signing_identity_exists || fail "'$SIGN_IDENTITY' not found. Run: make signing-setup"
  local probe out
  probe=$(mktemp)
  if ! out=$(codesign --force --sign "$SIGN_IDENTITY" "$probe" 2>&1); then
    # The keychain may have locked (reboot). Try the stored password once.
    local password
    if password=$(security find-generic-password -s "$SIGN_PASSWORD_SERVICE" -a "$USER" -w 2>/dev/null) \
      && security unlock-keychain -p "$password" "$SIGN_KEYCHAIN" 2>/dev/null \
      && out=$(codesign --force --sign "$SIGN_IDENTITY" "$probe" 2>&1); then
      :
    else
      rm -f "$probe"
      fail "test signing with '$SIGN_IDENTITY' failed: $out"
    fi
  fi
  rm -f "$probe"
}

# Reads codesign's own opinion: an adhoc signature has no Authority= line.
verify_signature() {
  local bin=$1 info authority identifier
  codesign --verify --strict "$bin" || fail "codesign --verify failed for $bin"
  info=$(codesign -dv --verbose=4 "$bin" 2>&1) || fail "codesign cannot read $bin"
  authority=$(printf '%s\n' "$info" | sed -n 's/^Authority=//p' | head -1)
  identifier=$(printf '%s\n' "$info" | sed -n 's/^Identifier=//p' | head -1)
  [ -n "$authority" ] || fail "$bin is adhoc-signed (no Authority= line)"
  [ "$authority" = "$SIGN_IDENTITY" ] || fail "$bin is signed by '$authority', not '$SIGN_IDENTITY'"
  [ "$identifier" = "$HELPER_ID" ] || fail "$bin has identifier '$identifier', not '$HELPER_ID'"
}

write_plist() {
  cat >"$HELPER_PLIST" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>$HELPER_ID</string>
  <key>ProgramArguments</key><array><string>$HELPER_BIN</string></array>
  <key>RunAtLoad</key><true/>
  <key>StartInterval</key><integer>$HELPER_INTERVAL_SECONDS</integer>
  <key>StandardOutPath</key><string>$HELPER_LOG</string>
  <key>StandardErrorPath</key><string>$HELPER_LOG</string>
</dict>
</plist>
PLIST
  plutil -lint "$HELPER_PLIST" >/dev/null || fail "generated plist is invalid"
}

echo "==> Checking the signing identity"
preflight_signing

echo "==> Building"
(cd "$root" && cargo build --release -p "$HELPER_NAME")

echo "==> Installing and signing"
mkdir -p "$HELPER_DIR" "$HELPER_LOG_DIR" "$(dirname "$HELPER_PLIST")"
# Replace the file (not overwrite in place) so a running copy is not corrupted.
cp "$root/target/release/$HELPER_NAME" "$HELPER_BIN.new"
codesign --force --options runtime --identifier "$HELPER_ID" --sign "$SIGN_IDENTITY" "$HELPER_BIN.new"
mv -f "$HELPER_BIN.new" "$HELPER_BIN"
verify_signature "$HELPER_BIN"

echo "==> Registering the launchd agent"
write_plist
launchctl bootout "gui/$(id -u)/$HELPER_ID" 2>/dev/null || true
launchctl bootstrap "gui/$(id -u)" "$HELPER_PLIST"

cat <<MSG

Installed. One manual step is left: give the helper Full Disk Access.

  1. In the System Settings pane that opens, click "+" and add this binary
     (in the file picker press Cmd+Shift+G and paste the path):

       $HELPER_BIN

  2. Make sure its switch is on.
  3. Run: make helper-run
  4. Check the log, which should say outcome=ok:  tail -n 3 "$HELPER_LOG"
     The Snapshot appears at:                     $HELPER_SNAPSHOT

MSG
open "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles" || true
