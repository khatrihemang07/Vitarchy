#!/usr/bin/env bash
# One-time: creates the self-signed "Vitarchy Dev" code-signing identity in its own
# keychain, used to sign the activity helper (docs/adr/0009).
#   usage: scripts/setup-signing.sh [--force]
#
# Refuses to run if the identity exists. macOS ties the helper's Full Disk Access
# grant to its signing identity, so a new identity revokes the grant and you would
# have to grant it again. --force deletes the keychain and creates a new identity.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
# shellcheck source=lib/helper.sh
. "$here/lib/helper.sh"

force=0
case "${1:-}" in
  "") ;;
  --force) force=1 ;;
  *) echo "usage: $0 [--force]" >&2; exit 2 ;;
esac

if signing_identity_exists && [ "$force" -eq 0 ]; then
  echo "'$SIGN_IDENTITY' already exists, so nothing was changed."
  echo "A new identity would revoke the helper's Full Disk Access grant, and you would"
  echo "have to grant it again. To recreate it anyway: $0 --force"
  exit 1
fi

echo "==> Creating the signing keychain"
security delete-keychain "$SIGN_KEYCHAIN" 2>/dev/null || true
password=$(openssl rand -base64 24)
security create-keychain -p "$password" "$SIGN_KEYCHAIN"
security unlock-keychain -p "$password" "$SIGN_KEYCHAIN"
security set-keychain-settings -u "$SIGN_KEYCHAIN" # no auto-lock

# On the search list beside the login keychain, so codesign finds it without -k.
security list-keychains -d user -s "$HOME/Library/Keychains/login.keychain-db" "$SIGN_KEYCHAIN"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

echo "==> Generating the certificate (10 years)"
openssl req -x509 -newkey rsa:2048 \
  -keyout "$work/key.pem" -out "$work/cert.pem" \
  -days 3650 -nodes -subj "/CN=$SIGN_IDENTITY" \
  -addext "keyUsage=critical,digitalSignature" \
  -addext "extendedKeyUsage=codeSigning" 2>/dev/null
openssl pkcs12 -export -out "$work/cert.p12" \
  -inkey "$work/key.pem" -in "$work/cert.pem" -passout pass:"$password"

security import "$work/cert.p12" -P "$password" -T /usr/bin/codesign -k "$SIGN_KEYCHAIN"
# Without this, every codesign run prompts for the keychain password.
security set-key-partition-list -S apple-tool:,apple: -s -k "$password" "$SIGN_KEYCHAIN" >/dev/null

# Keep the password in the login keychain so install-activity-helper.sh can unlock
# the signing keychain again after a reboot.
security add-generic-password -U -s "$SIGN_PASSWORD_SERVICE" -a "$USER" -w "$password"

echo "==> '$SIGN_IDENTITY' is installed in $SIGN_KEYCHAIN"
echo "    Next: make helper-install"
