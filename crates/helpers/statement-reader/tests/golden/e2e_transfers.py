"""What the end-to-end run asserts about Transfers after the import: each automatic rung found
some, at least one is partial, none breaks a rule, and an unlink survives a Read again. Counts only;
nothing private is printed, so a failure names the rung or the check, never a row."""

from __future__ import annotations

from e2e_env import E2E_DB, Api, check, psql

RUNGS = ("reference", "account_number", "bill_payment", "unique_amount")
TOLERANCE_PAISE = 5000


def rung_counts(stage_url: str) -> dict[str, int]:
    """How many Transfers each rung linked (every rung is present, 0 when none)."""
    rows = psql(stage_url, E2E_DB, "SELECT rung, count(*) FROM finance.transfers GROUP BY rung")
    found = {line.split("|")[0]: int(line.split("|")[1]) for line in rows.splitlines() if line}
    return {rung: found.get(rung, 0) for rung in (*RUNGS, "manual")}


def count(stage_url: str, sql: str) -> int:
    return int(psql(stage_url, E2E_DB, sql).strip())


# One row per Transfer: its debit's amount and what its credits add up to.
_SUMS = ("SELECT t.id, d.amount_paise, sum(x.amount_paise) AS credited, count(*) AS credits, "
         "bool_or(x.account_id = d.account_id) AS same_account "
         "FROM finance.transfers t JOIN finance.transactions d ON d.id = t.debit_id "
         "JOIN finance.transfer_credits c ON c.transfer_id = t.id "
         "JOIN finance.transactions x ON x.id = c.credit_id GROUP BY t.id, d.amount_paise, d.account_id")


def assert_rules(stage_url: str) -> None:
    """No Transfer breaks a rule: credits never exceed the debit by more than the tolerance, no
    credit sits on its debit's own Account, and only bill payments take several credits."""
    over = count(stage_url, f"SELECT count(*) FROM ({_SUMS}) s WHERE credited > amount_paise + {TOLERANCE_PAISE}")
    check(over == 0, f"no Transfer's credits exceed its debit by more than the tolerance ({over} do)")
    exact = count(stage_url, "SELECT count(*) FROM finance.transfers t JOIN ("
                  f"{_SUMS}) s ON s.id = t.id WHERE t.rung IN ('reference', 'account_number', 'unique_amount') "
                  "AND (credited <> amount_paise OR credits <> 1)")
    check(exact == 0, f"the rungs other than bill payment need exact amounts and one credit ({exact} break it)")
    same = count(stage_url, f"SELECT count(*) FROM ({_SUMS}) s WHERE same_account")
    check(same == 0, f"no Transfer stays on one Account ({same} do)")


def assert_unlink_survives_read_again(api: Api, stage_url: str) -> None:
    """Unlinks one Transfer, reads the Statement of its credit again and checks it stays unlinked."""
    picked = psql(stage_url, E2E_DB, "SELECT t.id, t.debit_id, c.credit_id FROM finance.transfers t "
                  "JOIN finance.transfer_credits c ON c.transfer_id = t.id WHERE t.rung = 'reference' ORDER BY t.id LIMIT 1").strip()
    if not picked:
        check(False, "a reference Transfer exists to unlink")
        return
    transfer, debit, credit = (int(n) for n in picked.split("|"))
    status, _ = api.call("DELETE", f"/api/finance/transfers/{transfer}")
    check(status == 204, f"unlinking a Transfer returned 204 (got {status})")
    statement = int(psql(stage_url, E2E_DB, "SELECT p.statement_id FROM finance.printings p JOIN finance.statements s "
                         f"ON s.id = p.statement_id AND s.status = 'confirmed' WHERE p.transaction_id = {credit} LIMIT 1").strip())
    status, _ = api.call("POST", f"/api/finance/statements/{statement}/read-again/confirm", {})
    check(status == 200, f"Read again after the unlink returned 200 (got {status})")
    linked = count(stage_url, f"SELECT count(*) FROM finance.transfers WHERE debit_id = {debit}")
    check(linked == 0, f"the unlinked Transfer stayed unlinked after Read again ({linked} Transfers on its debit)")
    remembered = count(stage_url, f"SELECT count(*) FROM finance.transfer_rejections WHERE debit_id = {debit} AND credit_id = {credit}")
    check(remembered == 1, "the unlink was remembered")


def assert_transfers(api: Api, stage_url: str, expected: dict) -> dict[str, int]:
    """The Transfers the import linked. Returns the count per rung. The private expectations may
    say how many partial Transfers the files hold (`partial_transfers`, default 0): a split bill
    payment only shows as partial while one of its cards' Statements is missing."""
    found = rung_counts(stage_url)
    partial = count(stage_url, f"SELECT count(*) FROM ({_SUMS}) s WHERE credited < amount_paise")
    split = count(stage_url, f"SELECT count(*) FROM ({_SUMS}) s WHERE credits > 1")
    print(f"  Transfers by rung: {found}; {partial} partial, {split} with several credits")
    for rung in ("reference", "account_number", "bill_payment"):
        check(found[rung] > 0, f"some Transfers linked by {rung} ({found[rung]})")
    wanted = expected.get("partial_transfers", 0)
    if wanted:
        check(partial >= wanted, f"at least {wanted} Transfers are partial ({partial})")
    else:
        print(f"  INFO  {partial} partial Transfers (none expected: the private files hold every card of a split payment)")
    assert_rules(stage_url)
    assert_unlink_survives_read_again(api, stage_url)
    return found
