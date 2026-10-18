"""Raw rows -> contract Transactions: dates, money, direction, references, add-on cards."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import date

from ..checks.figures import chain_breaks
from ..contract import Transaction
from ..segment.rows import RawRow
from ..vocabulary import Vocabulary
from .amounts import balance_paise, column_amount, parse_foreign, parse_money
from .dates import detect_format, parse_date
from .direction import Figures, decide
from .ids import resolve_ids
from .references import find_reference

NO_ID = ("", "-", "--")
INVERTED_KINDS = ("credit_card", "loan")


class NormaliseError(Exception):
    def __init__(self, code: str) -> None:
        super().__init__(code)
        self.code = code


@dataclass
class Built:
    transactions: list[Transaction] = field(default_factory=list)
    opening_paise: int | None = None
    skipped: int = 0


def _clean(text: str | None) -> str | None:
    return None if text is None or text.strip() in NO_ID else text.strip()


def _transaction(raw: RawRow, iso: str, direction: str, paise: int, balance: int | None,
                 vocab: Vocabulary, addon: str | None) -> Transaction:
    foreign = parse_foreign(raw.cells.get("foreign_amount", ""))
    return Transaction(
        date=iso, description=raw.description, amount_paise=paise, direction=direction,
        balance_paise=balance, foreign_amount=foreign[0] if foreign else None,
        foreign_currency=foreign[1] if foreign else None,
        reference=_clean(raw.cells.get("utr")) or find_reference(raw.description, vocab),
        addon_card=addon, source_id=_clean(raw.cells.get("source_id")))


def _figures(raw: RawRow, vocab: Vocabulary, previous: int | None) -> tuple[Figures, int | None]:
    balance = balance_paise(raw.cells.get("balance", ""), vocab)
    amount = parse_money(raw.cells["amount"], vocab) if "amount" in raw.cells else None
    debit = column_amount(raw.cells.get("debit", ""), vocab)
    credit = column_amount(raw.cells.get("credit", ""), vocab)
    return Figures(debit, credit, amount, raw.cells.get("type"), balance, previous), balance


def _pass(rows: list[RawRow], formats: str, vocab: Vocabulary, kind: str | None, opening: int | None,
          inverted: bool, primary: str | None) -> Built:
    built = Built(opening_paise=opening)
    negative_means = "credit" if kind in INVERTED_KINDS else "debit"
    previous = opening
    for raw in rows:
        figures, balance = _figures(raw, vocab, previous)
        decision = decide(figures, vocab, negative_means, use_balance=True, inverted=inverted)
        if balance is not None:
            previous = balance
        if decision is None:
            built.skipped += 1
            continue
        parsed = parse_date(raw.cells.get("date") or raw.cells.get("value_date", ""), formats, vocab)
        if parsed is None:
            raise NormaliseError("unreadable_dates")
        addon = raw.card if raw.card and raw.card != primary else None
        built.transactions.append(_transaction(raw, parsed.isoformat(), *decision, balance, vocab, addon))
    return built


def build_transactions(rows: list[RawRow], vocab: Vocabulary, kind: str | None,
                       period: tuple[date | None, date | None] | None = None,
                       opening_paise: int | None = None, mask: str | None = None) -> Built:
    """Transactions in printed order. An opening row sets the opening balance and is not a Transaction."""
    opening_rows = [r for r in rows if r.pseudo == "opening"]
    real = [r for r in rows if r.pseudo is None]
    resolve_ids(real)
    if opening_rows and opening_paise is None:
        opening_paise = balance_paise(opening_rows[0].cells.get("balance", ""), vocab)
    values = [r.cells.get("date") or r.cells.get("value_date", "") for r in real]
    fmt = detect_format(values, vocab, period) if real else None
    if real and fmt is None:
        raise NormaliseError("unreadable_dates")
    primary = mask or next((r.card for r in real if r.card), None)
    options = [_pass(real, fmt or "", vocab, kind, opening_paise, flag, primary) for flag in (False, True)]
    if all(o.transactions == options[0].transactions for o in options):
        return options[0]
    prefer_inverted = kind in INVERTED_KINDS

    def rank(index: int) -> tuple[int, bool]:
        breaks = chain_breaks(options[index].transactions, opening_paise, inverted=bool(index))[1]
        return breaks, bool(index) != prefer_inverted

    return options[min((0, 1), key=rank)]
