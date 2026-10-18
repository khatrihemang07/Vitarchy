"""What the end-to-end run asserts about a finished import: Accounts, payment-app joins, Printings
and the agreement of the two PhonePe orders. Counts only; nothing private is printed."""

from __future__ import annotations

from e2e_env import E2E_DB, Api, check, psql
from e2e_payment_apps import (
    find_account,
    ledger_counts,
    payment_app_totals,
    same_number,
    twin_printings,
    visible_digits,
)


def assert_accounts(api: Api, stage_url: str, expected: dict) -> dict[str, int]:
    """The Accounts, and the Transactions a bank or card Statement printed on each (a payment app
    adds its own on top). Returns the Transactions per expected Account, for comparing orders."""
    accounts = api.call("GET", "/api/finance/accounts")[1]
    bank = ledger_counts(lambda sql: psql(stage_url, E2E_DB, sql))
    expected_accounts = [tuple(a) for a in expected["accounts"]]
    unique = {tuple(u[:3]): u[3] for u in expected["unique_transactions"]}
    sbi_loans = expected["sbi_loans"]
    total = len(expected_accounts) + sbi_loans
    check(len(accounts) == total, f"{total} Accounts (got {len(accounts)})")
    by_account: dict[str, int] = {}
    for issuer, kind, digits in expected_accounts:
        spec = {"account": [issuer, kind, digits], "new": {"name": ""}}
        found = [a for a in accounts if issuer in a["issuer"].lower() and a["kind"] == kind and same_number(a["mask"], digits)]
        chosen = find_account(accounts, next((i for i in expected["instruments"].values() if i["account"] == [issuer, kind, digits]), spec))
        check(chosen is not None, f"an Account for {issuer} {kind} ending {digits} (got {len(found)} by number)")
        if chosen is None:
            continue
        mine = next(a for a in accounts if a["id"] == chosen)
        by_account[f"{issuer} {kind} {digits}"] = mine["transaction_count"]
        if (issuer, kind, digits) in unique:
            want = unique[(issuer, kind, digits)]
            check(bank.get(chosen, 0) == want, f"{issuer} {kind} {digits}: {want} Statement-printed Transactions (got {bank.get(chosen, 0)})")
    loans = [a for a in accounts if "state bank" in a["issuer"].lower() and a["kind"] == "loan"]
    check(len(loans) == sbi_loans, f"{sbi_loans} SBI loans (got {len(loans)})")
    return by_account


def assert_payment_apps(stage_url: str, expected: dict) -> dict[str, int]:
    """What the PhonePe files left: how many of their Transactions joined a bank or card
    Transaction, how many stand alone, and that nothing is counted twice."""
    totals = payment_app_totals(lambda sql: psql(stage_url, E2E_DB, sql))
    print(f"  PhonePe: {totals['app']} Transactions, {totals['joined']} joined a Statement's, {totals['alone']} only PhonePe's")
    linked = expected["phonepe_linked_before"]
    check(totals["joined"] >= linked, f"at least the {linked} previously linked payments are now Printings on Statement Transactions (got {totals['joined']})")
    check(totals["alone"] > 0, f"PhonePe rows without a Statement row became Transactions ({totals['alone']})")
    twins = twin_printings(lambda sql: psql(stage_url, E2E_DB, sql))
    check(twins == 0, f"no Transaction has two Printings from one Statement ({twins})")
    return totals


def assert_printings(stage_url: str, expected: dict, gains: dict[str, int]) -> None:
    """Counts only: every Transaction keeps its Printings, overlapping SBI files show the fuller text."""
    count = lambda sql: int(psql(stage_url, E2E_DB, sql).strip())  # noqa: E731
    transactions = count("SELECT count(*) FROM finance.transactions")
    printings = count("SELECT count(*) FROM finance.printings")
    orphans = count("SELECT count(*) FROM finance.transactions t WHERE NOT EXISTS "
                    "(SELECT 1 FROM finance.printings p WHERE p.transaction_id = t.id)")
    print(f"  {transactions} Transactions, {printings} Printings, {gains['confirmed_gain']} enriched by confirms")
    check(orphans == 0 and printings >= transactions, f"every Transaction has a Printing ({orphans} without)")
    identical = count("SELECT count(*) FROM (SELECT 1 FROM finance.printings GROUP BY transaction_id, date, "
                      "description, balance_paise, reference, source_id, foreign_amount, foreign_currency "
                      "HAVING count(*) > 1) twins")
    check(identical == 0, f"no two Printings of one Transaction are identical ({identical} such groups)")
    check(gains["previewed_gain"] > 0, f"previews said some rows would gain detail ({gains['previewed_gain']})")
    check(gains["previewed_gain"] == gains["confirmed_gain"],
          f"confirms enriched exactly what the previews promised ({gains['confirmed_gain']} of {gains['previewed_gain']})")
    both = expected["printed_by_pdf_and_xlsx"]
    sql = ("SELECT count(*), count(*) FILTER (WHERE t.description = (SELECT p.description FROM finance.printings p "
           "JOIN finance.statements s ON s.id = p.statement_id WHERE p.transaction_id = t.id AND s.format = 'xlsx' ORDER BY length(p.description) DESC LIMIT 1)) "
           "FROM finance.transactions t JOIN finance.accounts a ON a.id = t.account_id "
           f"WHERE a.issuer ILIKE '%{both['issuer']}%' AND a.kind = '{both['kind']}' "
           "AND (SELECT count(*) FROM finance.printings p WHERE p.transaction_id = t.id) >= 2 "
           "AND EXISTS (SELECT 1 FROM finance.printings p JOIN finance.statements s ON s.id = p.statement_id "
           "WHERE p.transaction_id = t.id AND s.format = 'pdf') "
           "AND EXISTS (SELECT 1 FROM finance.printings p JOIN finance.statements s ON s.id = p.statement_id "
           "WHERE p.transaction_id = t.id AND s.format = 'xlsx')")
    printed_both, show_xlsx = (int(n) for n in psql(stage_url, E2E_DB, sql).strip().split("|"))
    print(f"  {printed_both} printed by a PDF and an XLSX, {show_xlsx} show the XLSX's text")
    check(printed_both == both["count"], f"{both['count']} {both['issuer']} {both['kind']} Transactions have a PDF Printing and an XLSX Printing (got {printed_both})")
    # Informational, not asserted: the PDF's narration is often not a strict prefix of the
    # XLSX's (its last letter differs), so the strict-extension rule keeps the PDF text there.
    print(f"  INFO  {show_xlsx} of {printed_both} show the XLSX's fuller text")


def duplicate_account_pairs(stage_url: str) -> int:
    """How many pairs of Accounts share issuer, kind and visible digits (must be 0)."""
    rows = [r.split("|") for r in psql(stage_url, E2E_DB,
            "SELECT issuer, kind, coalesce(mask, '') FROM finance.accounts ORDER BY id").splitlines()]
    norm = lambda text: " ".join(text.lower().split())  # noqa: E731
    return sum(1 for i, a in enumerate(rows) for b in rows[i + 1:]
               if norm(a[0]) == norm(b[0]) and a[1] == b[1] and visible_digits(a[2]) and same_number(a[2], b[2]))


def compare_orders(orders: dict[str, dict]) -> None:
    """Confirming the PhonePe files first or last must leave the same Transactions on each Account."""
    print("both orders:")
    last, first = orders.get("last"), orders.get("first")
    check(bool(last and first), "both orders ran")
    if last and first:
        check(last["by_account"] == first["by_account"],
              f"the same Transactions per Account in both orders ({sum(last['by_account'].values())} vs {sum(first['by_account'].values())})")
        for key in ("transactions", "printings"):
            check(last["counts"][key] == first["counts"][key],
                  f"the same {key} count in both orders ({last['counts'][key]} vs {first['counts'][key]})")
