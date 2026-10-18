"""Payment-app rows -> PaymentRecords (when, what, how much, which UTR and instrument)."""

from __future__ import annotations

import re
from datetime import datetime, time

from ..contract import PaymentRecord
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


def instrument_details(label: str | None, vocab: Vocabulary) -> tuple[str | None, str | None]:
    """(issuer, trailing digits) recognised in a printed instrument such as "<BANK> R" or "XXXXXX4321"."""
    if not label:
        return None, None
    words = re.findall(r"[a-z]+", label.lower())
    issuer = next((name for name, shorts in vocab.instrument_issuers
                   if any(s in words for s in shorts)), None)
    digits = re.search(r"\d+$", label.strip())
    return issuer, digits.group(0) if digits else None


def build_payment_records(rows: list[RawRow], vocab: Vocabulary) -> list[PaymentRecord]:
    """Records in printed order. Date and time columns combine into `occurred_at`."""
    fmt = detect_format([r.cells.get("date", "") for r in rows], vocab)
    if rows and fmt is None:
        raise NormaliseError("unreadable_dates")
    records = []
    for raw in rows:
        day = parse_date(raw.cells["date"], fmt or "", vocab)
        decision = decide(Figures(amount=parse_money(raw.cells.get("amount", ""), vocab),
                                  type_text=raw.cells.get("type")), vocab)
        if day is None or decision is None:
            continue
        moment = parse_time(raw.cells.get("time", ""), vocab) or time()
        instrument = instrument_label(raw.cells.get("instrument", ""), vocab)
        issuer, digits = instrument_details(instrument, vocab)
        records.append(PaymentRecord(
            occurred_at=datetime.combine(day, moment).isoformat(), description=raw.description,
            amount_paise=decision[1], direction=decision[0],
            source_id=raw.cells.get("source_id") or None, utr=raw.cells.get("utr") or None,
            instrument=instrument, instrument_issuer=issuer, instrument_digits=digits))
    return records
