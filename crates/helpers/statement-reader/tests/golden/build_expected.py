"""Generates fixtures/private/expected/<file>.json from the research extractions in
fixtures/private/analysis/*_transactions.json and the counts in the research reports.

    uv run python -m tests.golden.build_expected

Expected is per file: outcome, kind, number of Statements and, per Statement, the Transaction
count, the sums of debit and credit paise, whether a period is printed, and the Checks that must come out as stated (only
those listed are asserted). Nothing here is committed from private/: this script holds no data.
"""

from __future__ import annotations

import json
from decimal import Decimal
from pathlib import Path

PRIVATE = Path(__file__).resolve().parents[2] / "fixtures" / "private"
ANALYSIS = PRIVATE / "analysis"
OUT = PRIVATE / "expected"

# Known facts from the reports: file names, the failing totals and the printed mask are private.
FACTS = json.loads((PRIVATE / "e2e_expected.json").read_text())["build_expected"]
FAILING_TOTALS = set(FACTS["failing_totals"])
FIRST_SBI_SHEET = tuple(FACTS["first_sbi_sheet"])  # (file, a prefix of the larger sheet)
SECOND_SBI_SHEET = FACTS["second_sbi_sheet"]
PHONEPE_SMALL = tuple(FACTS["phonepe_small"])  # (file, window start)
PHONEPE_LARGE = FACTS["phonepe_large"]
PHONEPE_REPORTED = {name: tuple(values) for name, values in FACTS["phonepe_reported"].items()}
OPTXN_PDF_MASK = FACTS["optxn_pdf_mask"]  # printed on both OpTransactionHistory PDFs


def paise(value: str | float | int) -> int:
    return int((Decimal(str(value)) * 100).to_integral_value())


def statement(rows: list[tuple[int, int]], checks: dict[str, str]) -> dict:
    """rows are (debit paise, credit paise) per Transaction."""
    return {"transactions": len(rows), "debit_paise": sum(d for d, _ in rows),
            "credit_paise": sum(c for _, c in rows), "checks": checks,
            "has_period": True}  # every real Statement prints a period (verified from the page text)


def statement_file(statements: list[dict]) -> dict:
    return {"outcome": "read", "kind": "statement", "statement_count": len(statements),
            "statements": statements}


def load(name: str):
    return json.loads((ANALYSIS / f"{name}_transactions.json").read_text())


def bank_pdfs() -> dict[str, dict]:
    expected = {}
    for entry in load("bank_pdfs"):
        statements = []
        for account in entry["accounts"]:
            rows = [(paise(t["debit"]), paise(t["credit"])) for t in account["transactions"]]
            checks = {"balance_chain": "passed"}
            if account["kind"].startswith("loan"):
                checks["summary_totals"] = "passed"
            if entry["file"].startswith("AccountStatement"):
                checks |= {"summary_totals": "passed", "row_counts": "passed"}
            statements.append(statement(rows, checks))
        expected[entry["file"]] = statement_file(statements)
    return expected


def card_files() -> dict[str, dict]:
    expected = {}
    for entry in load("icici_cc") + load("sbi_cc_and_statement18"):
        direction_key = "direction" if "direction" in entry["transactions"][0] else "dir"
        rows = [(paise(t["amount"]), 0) if t[direction_key] == "debit" else (0, paise(t["amount"]))
                for t in entry["transactions"]]
        outcome = "failed" if entry["file"] in FAILING_TOTALS else "passed"
        expected[entry["file"]] = statement_file([statement(rows, {"summary_totals": outcome})])
    return expected


def optxn_files() -> dict[str, dict]:
    by_file: dict[str, list[tuple[int, int]]] = {}
    for record in load("optxn")["records"]:
        by_file.setdefault(record["source"], []).append(
            (paise(record["withdrawal"]), paise(record["deposit"])))
    return {name: statement_file([statement(rows, {"balance_chain": "passed"})])
            for name, rows in by_file.items()}


def sheet_files() -> dict[str, dict]:
    data = load("csv_xlsx")
    rows = [(paise(r["debit"]), paise(r["credit"])) for r in data["xlsx_transactions"]]
    checks = {"balance_chain": "passed", "summary_totals": "passed", "row_counts": "passed"}
    return {SECOND_SBI_SHEET: statement_file([statement(rows, checks)]),
            FIRST_SBI_SHEET[0]: statement_file([statement(rows[:FIRST_SBI_SHEET[1]], checks)])}


def payment_history_files() -> dict[str, dict]:
    records = load("csv_xlsx")["phonepe_transactions"]
    expected = {}
    for name, start in ((PHONEPE_LARGE, ""), PHONEPE_SMALL):
        picked = [r for r in records if r["date"] >= start]
        debit = sum(paise(r["amount"]) for r in picked if r["direction"] == "out")
        credit = sum(paise(r["amount"]) for r in picked if r["direction"] != "out")
        count, want_debit, want_credit = PHONEPE_REPORTED[name]
        # The research counts and totals are the source of truth; refuse to write a mismatch.
        assert (len(picked), debit, credit) == (count, paise(want_debit), paise(want_credit)), name
        expected[name] = {"outcome": "read", "kind": "payment_history", "statement_count": 0,
                          "statements": [], "payment_records": {
                              "count": count, "debit_paise": debit, "credit_paise": credit}}
    return expected


def main() -> None:
    OUT.mkdir(exist_ok=True)
    expected = {**bank_pdfs(), **card_files(), **optxn_files(), **sheet_files(),
                **payment_history_files()}
    for name, body in expected.items():
        if name.startswith("OpTransactionHistory") and name.endswith(".pdf"):
            body["statements"][0]["mask"] = OPTXN_PDF_MASK
    on_disk = {p.name for p in (PRIVATE / "statements").iterdir() if p.is_file()}
    missing = on_disk - expected.keys()
    for name, body in expected.items():
        target = OUT / f"{name}.json"
        if target.exists():  # hand-verified description samples survive a rebuild
            body = body | {k: v for k, v in json.loads(target.read_text()).items() if k == "description_samples"}
        target.write_text(json.dumps(body, indent=1) + "\n")
    print(f"wrote {len(expected)} expected files; files without one: {len(missing)}")


if __name__ == "__main__":
    main()
