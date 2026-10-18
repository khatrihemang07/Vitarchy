"""Dates as printed -> `date`. The formats come from the vocabulary; each column settles on one."""

from __future__ import annotations

import re
from datetime import date, datetime, time, timedelta
from functools import lru_cache

from ..vocabulary import Vocabulary

MAX_DATE_WORDS = 4
MAX_DATE_LENGTH = 32
PERIOD_SLACK = timedelta(days=10)  # rows may sit just outside a printed period


def _clean(text: str, vocab: Vocabulary) -> str:
    text = " ".join(text.split())
    for alias, month in vocab.month_aliases.items():
        text = re.sub(rf"\b{alias}\b", month, text, flags=re.IGNORECASE)
    return text


@lru_cache(maxsize=65536)
def _strptime(text: str, fmt: str) -> date | None:
    try:
        return datetime.strptime(text, fmt).date()
    except ValueError:
        return None


def _plausible(text: str) -> bool:
    return any(c.isdigit() for c in text) and len(text) <= MAX_DATE_LENGTH


def parse_date(text: str, fmt: str, vocab: Vocabulary) -> date | None:
    return _strptime(_clean(text, vocab), fmt) if _plausible(text) else None


def parse_any(text: str, vocab: Vocabulary) -> tuple[date, str] | None:
    """The date in `text` and the first vocabulary format (day-first before month-first) that reads it."""
    if not _plausible(text):
        return None
    cleaned = _clean(text, vocab)
    for fmt in vocab.date_formats:
        found = _strptime(cleaned, fmt)
        if found:
            return found, fmt
    return None


def date_prefix_words(words: list[str], vocab: Vocabulary) -> int:
    """How many leading words form one date ("02 Jul 26" is 3); 0 when the text does not start with one."""
    for count in range(min(len(words), MAX_DATE_WORDS), 0, -1):
        if parse_any(" ".join(words[:count]), vocab):
            return count
    return 0


def detect_format(values: list[str], vocab: Vocabulary,
                  period: tuple[date | None, date | None] | None = None) -> str | None:
    """The one format that reads every value; day-first wins unless the period says otherwise."""
    cleaned = [_clean(v, vocab) for v in values if v.strip()]
    if not cleaned:
        return None
    fits = [fmt for fmt in vocab.date_formats if all(_strptime(v, fmt) for v in cleaned)]
    if len(fits) > 1 and period and period[0] and period[1]:
        low, high = period[0] - PERIOD_SLACK, period[1] + PERIOD_SLACK
        inside = [f for f in fits if all(low <= _strptime(v, f) <= high for v in cleaned)]  # type: ignore[operator]
        fits = inside or fits
    return fits[0] if fits else None


def parse_time(text: str, vocab: Vocabulary) -> time | None:
    for fmt in vocab.time_formats:
        try:
            return datetime.strptime(" ".join(text.split()), fmt).time()
        except ValueError:
            continue
    return None
