"""The repair scenario: duplicate Accounts made by bypassing the server's rule, then merged by
`vitarchy-server finance-merge-duplicates`. Counts only."""

from __future__ import annotations

import os
import subprocess
from urllib.parse import urlparse

from e2e_assertions import duplicate_account_pairs
from e2e_env import E2E_DB, ROOT, Api, check, counts, psql
from e2e_page_style import confirm_in_order, page_choice, upload_all
from e2e_payment_apps import visible_digits


def force_account(stage_url: str, proposed: dict) -> int | None:
    """Creates an Account by direct SQL, bypassing the server's rule, so duplicates can exist."""
    if not (proposed.get("issuer") and proposed.get("kind")):
        return None
    quote = lambda text: "'" + str(text).replace("'", "''") + "'"  # noqa: E731
    mask = proposed.get("mask") or ""
    digits = visible_digits(mask)
    last4 = quote(digits[-4:]) if len(digits) >= 4 else "NULL"
    sql = ("INSERT INTO finance.accounts (issuer, kind, mask, last4, name) VALUES "
           f"({quote(proposed['issuer'])}, {quote(proposed['kind'])}, {quote(mask) if mask else 'NULL'}, {last4}, 'Forced') RETURNING id")
    return int(psql(stage_url, E2E_DB, sql).splitlines()[0])


def merge_duplicates(stage_url: str) -> str:
    """Runs the repair subcommand against the e2e database; returns its counts-only line."""
    env = {**os.environ, "DATABASE_URL": urlparse(stage_url)._replace(path=f"/{E2E_DB}").geturl(),
           "VITARCHY_STAGE": "1", "RUST_LOG": "warn"}
    done = subprocess.run([str(ROOT / "target" / "debug" / "vitarchy-server"), "finance-merge-duplicates"],
                          env=env, capture_output=True, text=True)
    check(done.returncode == 0, "finance-merge-duplicates succeeded")
    return done.stdout.strip()


def run_repair(api: Api, stage_url: str, normal: dict[str, int], expected: dict) -> None:
    """Makes duplicate Accounts the way the old page did, then repairs them."""
    print("repair:")

    def choose(_, section: dict) -> dict:
        made = force_account(stage_url, section["proposed"])
        return {"account_id": made} if made else page_choice(section)

    refused = confirm_in_order(api, upload_all(api), choose, expected)
    check(refused == 0, "every confirm onto a forced Account returned 200")
    check(duplicate_account_pairs(stage_url) > 0, "the duplicates exist before the repair")
    before = counts(stage_url)
    print(f"  {merge_duplicates(stage_url)}")
    check(duplicate_account_pairs(stage_url) == 0, "no two Accounts share issuer, kind and digits after the repair")
    final = counts(stage_url)
    check(final == normal, f"same counts as the normal import ({final} vs {normal}; before {before})")
    merge_duplicates(stage_url)
    check(counts(stage_url) == final, "a second repair changes nothing")
