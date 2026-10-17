#!/usr/bin/env bash
# Names and paths shared by the activity-helper scripts (single source of truth).
# Sourced, not run. See docs/adr/0009-activity-via-signed-snapshot-helper.md.
#
# HELPER_NAME, HELPER_SNAPSHOT and HELPER_CLOCKIFY_SNAPSHOT are also written in Rust (crates cannot share them:
# helpers depend on nothing, modules only on core). scripts/test-guards.sh fails when
# they disagree, so change them in all places together.

SIGN_IDENTITY="Vitarchy Dev"
SIGN_KEYCHAIN="$HOME/Library/Keychains/vitarchy-signing.keychain-db"
# Where the keychain's password is kept (in the login keychain) so a locked
# signing keychain, for example after a reboot, can be unlocked again.
SIGN_PASSWORD_SERVICE="vitarchy-signing-keychain"

HELPER_NAME="activity-snapshot"
HELPER_ID="dev.vitarchy.activity-snapshot"
HELPER_DIR="$HOME/Library/Application Support/Vitarchy/bin"
HELPER_BIN="$HELPER_DIR/$HELPER_NAME"
HELPER_PLIST="$HOME/Library/LaunchAgents/$HELPER_ID.plist"
HELPER_LOG_DIR="$HOME/Library/Logs/Vitarchy"
HELPER_LOG="$HELPER_LOG_DIR/$HELPER_NAME.log"
HELPER_INTERVAL_SECONDS=28800 # every 8 hours, and at login (RunAtLoad)
HELPER_SNAPSHOT="$HOME/Library/Application Support/Vitarchy/activity/latest.sqlite"
HELPER_CLOCKIFY_SNAPSHOT="$HOME/Library/Application Support/Vitarchy/clockify/latest.sqlite"

# True if the signing identity is in a keychain on the search list.
# No -v: it reports 0 identities for a self-signed certificate, which codesign uses fine.
signing_identity_exists() {
  security find-identity -p codesigning 2>/dev/null | grep -qF "\"$SIGN_IDENTITY\""
}
