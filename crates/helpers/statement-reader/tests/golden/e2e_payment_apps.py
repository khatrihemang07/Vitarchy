"""The payment-app (PhonePe) parts of the end-to-end run: which Account each instrument section is
confirmed to, and the counts of what the import left. Counts only; nothing private is printed.

The private expectations (`e2e_expected.json`) map each instrument as printed to the Account it
stands for (issuer contains, kind, trailing digits) and to the new Account to create when that
Account does not exist yet, e.g. {"INSTRUMENT": {"account": [...], "new": {...}}}."""

from __future__ import annotations

import re
from collections.abc import Callable

Query = Callable[[str], str]


def visible_digits(mask: str | None) -> str:
    match = re.search(r"\d+$", (mask or "").strip())
    return match.group(0) if match else ""


def same_number(a: str | None, b: str | None) -> bool:
    da, db = visible_digits(a), visible_digits(b)
    if len(da) >= 2 and len(db) >= 2:
        n = min(len(da), len(db))
        return da[-n:] == db[-n:]
    return bool(a) and "".join((a or "").upper().split()) == "".join((b or "").upper().split())


def find_account(accounts: list[dict], spec: dict) -> int | None:
    """The oldest Account that is the instrument's expected Account, or the one this run created
    for it when it did not exist yet (found by the new Account's name)."""
    issuer, kind, digits = spec["account"]
    named = spec["new"]["name"]
    fits = [a for a in accounts if issuer in a["issuer"].lower() and a["kind"] == kind and same_number(a["mask"], digits)]
    fits = fits or [a for a in accounts if a["name"] == named]
    return min((a["id"] for a in fits), default=None)


def instrument_choice(accounts: list[dict], section: dict, instruments: dict) -> dict:
    """The choice for one payment-app section: its proposal when one Account was matched, else
    the expected Account if it exists, else a new Account made from the expectations."""
    if section.get("matched_account"):
        return {}
    spec = instruments[section["proposed"]["mask"]]
    existing = find_account(accounts, spec)
    return {"account_id": existing} if existing is not None else {"new_account": spec["new"]}


def reuse_instrument_account(accounts: list[dict], section: dict, instruments: dict) -> dict | None:
    """For a Statement section that matched no Account: the Account a payment app already created
    for this one (a card an instrument could only name, not number), as the person would pick it."""
    if section.get("matched_account"):
        return None
    proposed = section["proposed"]
    for spec in instruments.values():
        issuer, kind, digits = spec["account"]
        if (issuer in (proposed.get("issuer") or "").lower() and proposed.get("kind") == kind
                and same_number(proposed.get("mask"), digits)):
            made = [a["id"] for a in accounts if a["name"] == spec["new"]["name"]]
            if made:
                return {"account_id": made[0]}
    return None


def ledger_counts(query: Query) -> dict[int, int]:
    """Transactions per Account that a bank or card Statement printed (a live Printing of one)."""
    rows = query(
        "SELECT t.account_id, count(*) FROM finance.transactions t WHERE EXISTS ("
        "SELECT 1 FROM finance.printings p JOIN finance.statements s ON s.id = p.statement_id "
        "WHERE p.transaction_id = t.id AND p.gone_at IS NULL AND s.kind = 'statement') GROUP BY 1").splitlines()
    return {int(a): int(n) for a, n in (row.split("|") for row in rows)}


def payment_app_totals(query: Query) -> dict[str, int]:
    """Transactions a payment app printed: all, those a bank or card Statement also printed
    (the app's Printing joined an existing Transaction), and those only the app printed."""
    sql = (
        "WITH kinds AS (SELECT p.transaction_id, bool_or(s.kind = 'payment_history') AS app, "
        "bool_or(s.kind = 'statement') AS ledger FROM finance.printings p "
        "JOIN finance.statements s ON s.id = p.statement_id WHERE p.gone_at IS NULL GROUP BY 1) "
        "SELECT count(*) FILTER (WHERE app), count(*) FILTER (WHERE app AND ledger), "
        "count(*) FILTER (WHERE app AND NOT ledger) FROM kinds")
    app, joined, alone = (int(n) for n in query(sql).strip().split("|"))
    return {"app": app, "joined": joined, "alone": alone}


def twin_printings(query: Query) -> int:
    """Transactions with two Printings from one Statement (must be 0)."""
    return int(query("SELECT count(*) FROM (SELECT 1 FROM finance.printings GROUP BY transaction_id, "
                     "statement_id HAVING count(*) > 1) twins").strip())


def section_choices(api, current: dict, expected: dict) -> list[dict]:
    """One choice per section of a preview as it stands now: the proposal when it is matched, a new
    Account when a Statement's proposal names an issuer and kind, and for a payment app the
    expectations' Account for the instrument (see `e2e_payment_apps`)."""
    accounts = api.call("GET", "/api/finance/accounts")[1]
    instruments = expected["instruments"]
    choices = []
    for section in current.get("sections") or []:
        proposed = section["proposed"]
        if current["kind"] == "payment_history":
            choices.append(instrument_choice(accounts, section, instruments))
        elif reuse := reuse_instrument_account(accounts, section, instruments):
            choices.append(reuse)
        elif proposed.get("issuer") and proposed.get("kind"):
            choices.append({})
        else:
            choices.append({"new_account": {"issuer": proposed.get("issuer") or "Unknown",
                                            "kind": proposed.get("kind") or "savings", "name": "Unnamed"}})
    return choices
