"""Reads the value part of a label/value pair: money, counts, masks and statement periods."""

from __future__ import annotations

from datetime import date

from ..normalize.amounts import balance_paise
from ..normalize.dates import detect_format, parse_date
from ..vocabulary import Vocabulary

MIN_MASK_DIGITS = 9  # a bare run of digits this long is an unmasked number as printed
MAX_MASK_DIGITS = 18  # longer runs are references, not account or card numbers
MASK_CHARS = "xX*•"


def _looks_like_money(text: str, vocab: Vocabulary) -> bool:
    """A figure printed as money has paise or a currency mark; a bare integer is a count or points."""
    lowered = text.lower()
    return "." in text or any(mark and mark in lowered for mark in vocab.currency_marks)


def money_value(text: str, vocab: Vocabulary) -> int | None:
    return balance_paise(text, vocab) if _looks_like_money(text, vocab) else None


def total_value(text: str, vocab: Vocabulary) -> int | None:
    """A total of money: always positive; a printed '-' or '+' is only formatting."""
    value = money_value(text, vocab)
    return None if value is None else abs(value)


def count_value(text: str, vocab: Vocabulary) -> int | None:
    text = text.strip()
    return int(text) if text.isdigit() else None


def _is_long_digits(token: str) -> bool:
    return token.isdigit() and MIN_MASK_DIGITS <= len(token) <= MAX_MASK_DIGITS


def mask_value(text: str, vocab: Vocabulary) -> str | None:
    """The first run of masked-number tokens in `text`, as printed ("xxxx xxxx xxxx 9876")."""
    run: list[str] = []
    for token in text.replace(":", " ").split():
        masked = vocab.mask_token.match(token) is not None
        long_digits = _is_long_digits(token)
        anchor = any(c in MASK_CHARS for c in token) or long_digits
        if (masked or long_digits) and (run or anchor):
            run.append(token)
        elif run:
            break
    anchored = any(any(c in MASK_CHARS for c in t) or _is_long_digits(t) for t in run)
    return " ".join(run) if anchored else None


def period_value(text: str, vocab: Vocabulary) -> tuple[date | None, date | None] | None:
    """(start, end) from "June 29, 2026 to July 28, 2026" or "01-09-2026 To 30-09-2026"."""
    lowered = text.lower()
    for separator in vocab.period_separators:
        if separator in lowered:
            index = lowered.index(separator)
            left, right = text[:index], text[index + len(separator):]
            left, right = left.strip(" :"), right.strip(" :.")
            fmt = detect_format([left, right], vocab)  # one format for both ends
            start = parse_date(left, fmt, vocab) if fmt else None
            end = parse_date(right, fmt, vocab) if fmt else None
            if start and end:
                return start, end
    return None


def first_mask(text: str, vocab: Vocabulary) -> str | None:
    """The first masked card or account number printed anywhere in `text`."""
    for pattern in (vocab.masked_number, vocab.masked_account):
        match = pattern.search(text)
        if match:
            return match.group(0)
    return None
