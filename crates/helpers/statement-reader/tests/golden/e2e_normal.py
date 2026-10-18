"""The normal scenario of the end-to-end run: upload every private file and confirm them with the
PhonePe files first or last, then re-upload and Read again. Counts only."""

from __future__ import annotations

import json

from e2e_assertions import assert_accounts, assert_payment_apps, assert_printings
from e2e_env import FILE_COUNT, PRIVATE, Api, check, counts
from e2e_payment_apps import section_choices


def confirm_all(api: Api, previews: list[dict], expected: dict, phonepe: str) -> dict[str, int]:
    """Confirms every file, the PhonePe files `phonepe` ("first" or "last"); returns what the
    previews promised and what the confirms did, so the two can be compared. A PhonePe file with a
    section the server could not match is left out of that comparison: its preview cannot know
    the Account the person will pick."""
    refused = 0
    totals = {"previewed_gain": 0, "confirmed_gain": 0}
    ordered = sorted(previews, key=lambda p: p["kind"] == "payment_history", reverse=phonepe == "first")
    for preview in ordered:
        # The preview as it is just before this confirm, against what earlier confirms stored.
        _, current = api.call("GET", f"/api/finance/statements/{preview['id']}")
        sections = section_choices(api, current, expected)
        previewable = all(s.get("matched_account") for s in current["sections"]) or current["kind"] != "payment_history"
        gain = sum(s["enriched_count"] for s in current.get("sections", []))
        status, reply = api.call("POST", f"/api/finance/statements/{preview['id']}/confirm", {"sections": sections})
        if status != 200:
            check(False, f"confirm statement {preview['id']} returned 200 (got {status}, code {(reply or {}).get('code')})")
            refused += 1
        elif previewable:
            totals["previewed_gain"] += gain
            totals["confirmed_gain"] += reply["enriched"]
    check(refused == 0, f"every confirm returned 200 ({len(previews) - refused}/{len(previews)})")
    return totals


def run(api: Api, stage_url: str, expected: dict, phonepe: str) -> dict:
    """The normal scenario, the PhonePe files confirmed `phonepe` ("first" or "last"); returns its
    final counts for the others to match."""
    passwords = json.loads((PRIVATE / "passwords.json").read_text())
    for label, secrets in passwords.items():
        for index, secret in enumerate(secrets):
            status, _ = api.call("POST", "/api/finance/statement-passwords",
                                 {"label": f"{label.split('_format')[0]}-{index}", "secret": secret})
            if status not in (200, 201):
                check(False, f"saved password {label.split('_format')[0]}-{index} (got {status})")
    files = sorted(p for p in (PRIVATE / "statements").iterdir() if p.is_file())
    status, previews = api.upload(files)
    print(f"uploaded {len(files)} files, status {status}")
    check(len(previews) == FILE_COUNT and all(p["outcome"] == "read" for p in previews),
          f"all {FILE_COUNT} files read ({sum(p['outcome'] == 'read' for p in previews)}/{len(previews)})")
    failed = [(s["proposed"].get("issuer") or "") for p in previews for s in p.get("sections") or []
              for c in s["checks"] if c["outcome"] == "failed"]
    issuer = expected["failed_check"]["issuer"]
    check(len(failed) == 1 and issuer in failed[0].lower(), f"exactly 1 failed Check, in the expected file (got {len(failed)})")
    gains = confirm_all(api, previews, expected, phonepe)
    by_account = assert_accounts(api, stage_url, expected)
    assert_payment_apps(stage_url, expected)
    assert_printings(stage_url, expected, gains)
    final = counts(stage_url)
    print(f"  final counts: {final}")
    _, again = api.upload(files)
    dups = sum(1 for p in again if p.get("duplicate_file"))
    check(dups == FILE_COUNT, f"re-upload: {dups}/{len(again)} duplicate_file")
    adds = vanished = new_printings = enriched = errors = 0
    for preview in previews:
        status, reply = api.call("POST", f"/api/finance/statements/{preview['id']}/read-again")
        errors += status != 200
        for section in (reply or {}).get("sections", []):
            adds += section["add_count"]
            vanished += section["no_longer_found_count"]
            new_printings += section["new_printing_count"]
            enriched += section["enriched_count"]
    check(errors == 0, f"every Read again returned 200 ({len(previews) - errors}/{len(previews)})")
    check(adds == 0 and vanished == 0 and new_printings == 0 and enriched == 0,
          f"Read again: {adds} to add, {new_printings} Printings to add ({enriched} gaining detail), {vanished} no longer found")
    return {"counts": final, "by_account": by_account}
