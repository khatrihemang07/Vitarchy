"""The page-style scenario: confirm with the choices the web page builds from the upload-time
previews, in upload order. Counts only."""

from __future__ import annotations

import json

from e2e_assertions import duplicate_account_pairs
from e2e_env import PRIVATE, Api, check, counts
from e2e_payment_apps import section_choices


def page_choice(section: dict) -> dict:
    """The choice the web page sends for a section, from `sectionChoice.initialDraft`."""
    if section.get("matched_account"):
        return {"account_id": section["matched_account"]["id"]}
    if section.get("candidate_accounts"):
        return {"account_id": None}  # the page makes the person choose; sent as "use the match"
    proposed = section["proposed"]
    return {"new_account": {"issuer": proposed.get("issuer") or "Unknown",
                            "kind": proposed.get("kind") or "savings", "name": "Unnamed"}}


def confirm_in_order(api: Api, previews: list[dict], choose, expected: dict) -> int:
    """Confirms Statements then payment histories in upload order. A Statement section's choice is
    built by `choose` from the preview as it was at upload; a payment app's from its preview as it
    stands then, as the page would show it. Returns how many confirms were refused."""
    refused = 0
    for preview in sorted(previews, key=lambda p: p["kind"] == "payment_history"):
        if preview["kind"] == "payment_history":
            sections = section_choices(api, api.call("GET", f"/api/finance/statements/{preview['id']}")[1], expected)
        else:
            sections = [choose(preview, section) for section in preview.get("sections") or []]
        status, reply = api.call("POST", f"/api/finance/statements/{preview['id']}/confirm", {"sections": sections})
        if status != 200:
            print(f"    confirm statement {preview['id']} returned {status}, code {(reply or {}).get('code')}")
            refused += 1
    return refused


def upload_all(api: Api) -> list[dict]:
    """Saves the Statement passwords, uploads every private file and returns the upload-time previews."""
    passwords = json.loads((PRIVATE / "passwords.json").read_text())
    for label, secrets in passwords.items():
        for index, secret in enumerate(secrets):
            api.call("POST", "/api/finance/statement-passwords", {"label": f"{label.split('_format')[0]}-{index}", "secret": secret})
    _, previews = api.upload(sorted(p for p in (PRIVATE / "statements").iterdir() if p.is_file()))
    return previews


def run_page_style(api: Api, stage_url: str, normal: dict[str, int], expected: dict) -> None:
    """Confirms as the page does: choices from the upload-time previews, in upload order."""
    print("page-style import:")
    refused = confirm_in_order(api, upload_all(api), lambda _, section: page_choice(section), expected)
    check(refused == 0, "every page-style confirm returned 200")
    check(duplicate_account_pairs(stage_url) == 0, "no two Accounts share issuer, kind and digits")
    final = counts(stage_url)
    check(final == normal, f"same counts as the normal import ({final} vs {normal})")
