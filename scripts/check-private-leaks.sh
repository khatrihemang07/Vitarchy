#!/usr/bin/env bash
# Fails when a committed (or intent-to-add) file holds an identifier from the private real Statements.
#   usage: scripts/check-private-leaks.sh      (run by `make check`)
# Skips with a notice when the denylist is absent, as in CI, which has no private data. Build the
# denylist with: cd crates/helpers/statement-reader && uv run python -m tests.privacy.build_denylist
set -euo pipefail
exec python3 "$(dirname "$0")/check_private_leaks.py"
