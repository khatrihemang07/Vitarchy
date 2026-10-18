"""Compares a helper Result with the expected counts for a real file. Output is numbers and enum
names only: never descriptions, amounts of single Transactions, names or account numbers."""

from __future__ import annotations

import re


def summarise(result: dict) -> dict:
    """The comparable shape of a Result: outcome, kind, per-Statement counts, sums and checks."""
    statements = [{
        "transactions": len(s["transactions"]),
        "debit_paise": _sum(s["transactions"], "debit"),
        "credit_paise": _sum(s["transactions"], "credit"),
        "checks": {c["name"]: c["outcome"] for c in s["checks"]},
        "mask": (s.get("account") or {}).get("mask"),
        "has_period": all((s.get("period") or {}).get(end) for end in ("start", "end")),
    } for s in result.get("statements", [])]
    records = result.get("payment_records", [])
    return {
        "outcome": result.get("outcome"), "kind": result.get("kind"),
        "statement_count": len(statements), "statements": statements,
        "payment_records": {"count": len(records), "debit_paise": _sum(records, "debit"),
                            "credit_paise": _sum(records, "credit")},
    }


def _sum(rows: list[dict], direction: str) -> int:
    return sum(r["amount_paise"] for r in rows if r["direction"] == direction)


AMOUNT = r"[₹`]?\d[\d,]*\.\d{2}(?:\s?(?:CR|DR|Cr|Dr))?"
BARE_AMOUNT = re.compile(rf"^{AMOUNT}(?:\s|$)|(?:^|\s){AMOUNT}$")


def description_failures(expected: dict, result: dict) -> list[str]:
    """Codes only (an index, never the text): invariants on every row plus the verified samples."""
    groups = [(f"s{i}", s["transactions"]) for i, s in enumerate(result.get("statements", []))]
    groups.append(("p", result.get("payment_records", [])))
    failures = []
    for label, rows in groups:
        empty = [i for i, r in enumerate(rows) if not r["description"].strip()]
        edged = [i for i, r in enumerate(rows) if BARE_AMOUNT.search(r["description"].strip())]
        failures += [f"{label}.row[{i}] empty description" for i in empty[:3]]
        failures += [f"{label}.row[{i}] description starts or ends with an amount" for i in edged[:3]]
    lookup = dict(groups)
    for number, sample in enumerate(expected.get("description_samples", [])):
        rows = lookup.get(sample["where"], [])
        if sample["row"] >= len(rows) or rows[sample["row"]]["description"] != sample["text"]:
            failures.append(f"sample[{number}] ({sample['where']}.row[{sample['row']}]) differs")
    return failures


def compare(expected: dict, result: dict) -> list[str]:
    """Failure codes (empty when the file passes), each saying what differed and by how much."""
    actual = summarise(result)
    failures = [f"{key}: {actual[key]} != {expected[key]}"
                for key in ("outcome", "kind", "statement_count") if actual[key] != expected[key]]
    for i, want in enumerate(expected.get("statements", [])):
        if i >= len(actual["statements"]):
            break
        failures += _statement_failures(i, want, actual["statements"][i])
    if "payment_records" in expected:
        for key, value in expected["payment_records"].items():
            if actual["payment_records"][key] != value:
                failures.append(f"payment_records.{key}: {actual['payment_records'][key]} != {value}")
    return failures + description_failures(expected, result)


def _statement_failures(index: int, want: dict, got: dict) -> list[str]:
    failures = [f"s{index}.{key}: {got[key]} != {want[key]}"
                for key in ("transactions", "debit_paise", "credit_paise") if got[key] != want[key]]
    if "mask" in want and got["mask"] != want["mask"]:
        failures.append(f"s{index}.mask differs")
    if want.get("has_period") and not got["has_period"]:
        failures.append(f"s{index}.period: start or end missing")
    for name, outcome in want.get("checks", {}).items():
        if got["checks"].get(name) != outcome:
            failures.append(f"s{index}.check.{name}: {got['checks'].get(name)} != {outcome}")
    return failures
