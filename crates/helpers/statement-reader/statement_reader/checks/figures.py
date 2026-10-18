"""The Checks of a Statement: does it agree with itself and with what it printed?"""

from __future__ import annotations

from ..contract import Check, Summary, Transaction

ROUNDING_TOLERANCE = 99  # paise: some issuers print the closing balance in whole rupees


def _step(transaction: Transaction, inverted: bool) -> int:
    credit = transaction.direction == "credit"
    return transaction.amount_paise * (1 if credit != inverted else -1)


def chain_breaks(transactions: list[Transaction], opening: int | None,
                 inverted: bool) -> tuple[int, int]:
    """(comparisons made, breaks) walking the running balance in printed order."""
    previous, compared, breaks = opening, 0, 0
    for transaction in transactions:
        if transaction.balance_paise is None:
            previous = None
            continue
        if previous is not None:
            compared += 1
            breaks += previous + _step(transaction, inverted) != transaction.balance_paise
        previous = transaction.balance_paise
    return compared, breaks


def balance_chain(transactions: list[Transaction], opening: int | None) -> Check:
    """Each row's balance follows from the one before; tries a savings and an inverted (loan) reading."""
    normal, inverted = (chain_breaks(transactions, opening, flag) for flag in (False, True))
    rows, breaks = min(normal, inverted, key=lambda result: result[1])
    if rows == 0:
        return Check("balance_chain", "skipped", {"rows": 0, "breaks": 0, "polarity": "normal"})
    polarity = "normal" if normal[1] <= inverted[1] else "inverted"
    return Check("balance_chain", "passed" if breaks == 0 else "failed",
                 {"rows": rows, "breaks": breaks, "polarity": polarity})


def _sums(transactions: list[Transaction]) -> tuple[int, int]:
    debit = sum(t.amount_paise for t in transactions if t.direction == "debit")
    credit = sum(t.amount_paise for t in transactions if t.direction == "credit")
    return debit, credit


def _identity_holds(opening: int, closing: int, debit: int, credit: int, charges: int,
                    kind: str | None) -> bool:
    normal = abs(opening + credit - debit - closing) <= ROUNDING_TOLERANCE
    inverted = abs(opening - credit + debit + charges - closing) <= ROUNDING_TOLERANCE
    if kind in ("credit_card", "loan"):
        return inverted
    return normal if kind == "savings" else normal or inverted


def summary_totals(transactions: list[Transaction], summary: Summary, kind: str | None,
                   charges: int = 0) -> Check:
    """Sums equal the printed totals, and opening and closing follow from them."""
    debit, credit = _sums(transactions)
    results: list[bool] = []
    if summary.total_debit_paise is not None:
        results.append(summary.total_debit_paise == debit)
    if summary.total_credit_paise is not None:
        results.append(summary.total_credit_paise == credit)
    if summary.opening_paise is not None and summary.closing_paise is not None:
        results.append(_identity_holds(summary.opening_paise, summary.closing_paise, debit, credit,
                                       charges, kind))
    if not results:
        return Check("summary_totals", "skipped", {"compared": 0, "mismatched": 0})
    bad = results.count(False)
    return Check("summary_totals", "passed" if bad == 0 else "failed",
                 {"compared": len(results), "mismatched": bad})


def row_counts(transactions: list[Transaction], summary: Summary) -> Check:
    """The numbers of debit and credit rows equal the printed counts."""
    printed = {"debit": summary.debit_count, "credit": summary.credit_count}
    results = [sum(t.direction == direction for t in transactions) == count
               for direction, count in printed.items() if count is not None]
    if not results:
        return Check("row_counts", "skipped", {"compared": 0, "mismatched": 0})
    bad = results.count(False)
    return Check("row_counts", "passed" if bad == 0 else "failed",
                 {"compared": len(results), "mismatched": bad})
