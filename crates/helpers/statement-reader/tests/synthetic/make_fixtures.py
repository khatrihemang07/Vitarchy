"""Regenerates the committed synthetic fixtures: python -m tests.synthetic.make_fixtures

Every file is fake (invented names and numbers). Each hard trait has one file; `files/expected/`
holds the Transactions planted in it, and `files/manifest.json` the facts the tests assert.
"""

from __future__ import annotations

import json
from pathlib import Path

from .layouts_accounts import card_with_addon, consolidated
from .layouts_ledger import borderless_wrapped, encrypted_pdf, ruled_simple, side_panel
from .layouts_pages import centred_narration, headerless, two_line_header_pages
from .layouts_robust import balance_first, month_first_csv, signed_amount
from .layouts_sheets import encrypted_xlsx, payment_history_csv, xls_with_ghost_row

FILES = Path(__file__).parent / "files"
BUILDERS = [
    ruled_simple, borderless_wrapped, side_panel, two_line_header_pages, headerless,
    consolidated, card_with_addon, centred_narration, encrypted_pdf, encrypted_xlsx,
    xls_with_ghost_row, payment_history_csv, balance_first, signed_amount, month_first_csv,
]


def main() -> None:
    (FILES / "expected").mkdir(parents=True, exist_ok=True)
    manifest = {}
    for build in BUILDERS:
        built = build(FILES)
        stem = Path(built.filename).stem
        (FILES / "expected" / f"{stem}.json").write_text(
            json.dumps(built.expected, indent=2, ensure_ascii=False) + "\n")
        manifest[built.filename] = built.facts
    (FILES / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    print(f"wrote {len(BUILDERS)} fixtures to {FILES}")


if __name__ == "__main__":
    main()
