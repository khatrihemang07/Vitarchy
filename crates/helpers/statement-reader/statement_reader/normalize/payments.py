"""Payment-app rows -> one Statement per instrument (the account the app paid from or into)."""

from __future__ import annotations

import re
from datetime import date

from ..contract import Account, Period, Statement, Summary, Transaction
from ..segment.rows import RawRow
from ..vocabulary import Vocabulary
from .amounts import parse_money
from .dates import detect_format, parse_date, parse_time
from .direction import Figures, decide
from .rows import NormaliseError


def instrument_label(text: str, vocab: Vocabulary) -> str | None:
    """The funding account as printed, without the leading words ("Paid by ")."""
    text = " ".join(text.split())
    for prefix in sorted(vocab.payment_history["instrument_prefixes"], key=len, reverse=True):
        if text.lower().startswith(prefix):
            text = text[len(prefix):].strip()
            break
    return text or None


def instrument_issuer(label: str | None, vocab: Vocabulary) -> str | None:
    """The issuer the vocabulary recognises in a printed instrument such as "<BANK> R", else None."""
    words = re.findall(r"[a-z]+", (label or "").lower())
    return next((name for name, shorts in vocab.instrument_issuers
                 if any(s in words for s in shorts)), None)


def _transaction(raw: RawRow, day: date, decision: tuple[str, int], vocab: Vocabulary) -> Transaction:
    moment = parse_time(raw.cells.get("time", ""), vocab)
    return Transaction(
        date=day.isoformat(), time=moment.strftime("%H:%M:%S") if moment else None,
        description=raw.description,
        amount_paise=decision[1], direction=decision[0], balance_paise=None,
        reference=raw.cells.get("utr") or None, source_id=raw.cells.get("source_id") or None)


def payment_history_statements(rows: list[RawRow], vocab: Vocabulary) -> list[Statement]:
    """One Statement per distinct instrument in first-seen order, rows in printed order. The app
    prints no balances or totals, so each summary is empty and no Check applies."""
    fmt = detect_format([r.cells.get("date", "") for r in rows], vocab)
    if rows and fmt is None:
        raise NormaliseError("unreadable_dates")
    sections: dict[str | None, list[Transaction]] = {}
    for raw in rows:
        day = parse_date(raw.cells["date"], fmt or "", vocab)
        decision = decide(Figures(amount=parse_money(raw.cells.get("amount", ""), vocab),
                                  type_text=raw.cells.get("type")), vocab)
        if day is None or decision is None:
            continue
        label = instrument_label(raw.cells.get("instrument", ""), vocab)
        sections.setdefault(label, []).append(_transaction(raw, day, decision, vocab))
    return [Statement(Account(instrument_issuer(label, vocab), None, label),
                      Period(min(t.date for t in found), max(t.date for t in found)), Summary(), found, [])
            for label, found in sections.items()]
