"""Amounts as printed -> integer paise, exactly (Decimal, never float)."""

from __future__ import annotations

import re
from dataclasses import dataclass
from decimal import ROUND_HALF_UP, Decimal
from functools import lru_cache

from ..vocabulary import Vocabulary

OPERATOR = re.compile(r"^[-−–+]\s+")  # "- 1,696" in a summary box is formatting, not a sign
NUMBER = re.compile(
    r"^(?P<sign>[-−–+]?)\s*(?P<open>\()?\s*(?P<num>\d(?:[\d,]*\d)?(?:\.\d+)?|\.\d+)\s*(?P<close>\))?"
    r"\s*(?P<suffix>[A-Za-z]{1,2})?$")
NEGATIVE_SIGNS = "-−–"


@dataclass(frozen=True)
class Money:
    paise: int  # magnitude, never negative
    negative: bool = False
    direction: str | None = None  # "debit" or "credit" when a Cr/Dr/C/D suffix was printed


def _strip_marks(text: str, vocab: Vocabulary) -> str:
    for mark in sorted(vocab.currency_marks, key=len, reverse=True):
        if mark:
            text = re.sub(re.escape(mark), "", text, flags=re.IGNORECASE)
    return " ".join(text.split())


def _suffix_direction(suffix: str, vocab: Vocabulary) -> str | None:
    word = suffix.lower()
    if word in vocab.directions["credit_suffixes"]:
        return "credit"
    if word in vocab.directions["debit_suffixes"]:
        return "debit"
    return None


def parse_money(text: str, vocab: Vocabulary) -> Money | None:
    """The amount in `text`, or None when it is not one (blank, "-", a date, words)."""
    return _parse_money(text, vocab)


@lru_cache(maxsize=65536)
def _parse_money(text: str, vocab: Vocabulary) -> Money | None:
    cleaned = OPERATOR.sub("", _strip_marks(text.strip(), vocab))
    match = NUMBER.match(cleaned)
    if not match or bool(match["open"]) != bool(match["close"]):
        return None
    direction = None
    if match["suffix"]:
        direction = _suffix_direction(match["suffix"], vocab)
        if direction is None:
            return None
    value = Decimal(match["num"].replace(",", "")) * 100
    negative = bool(match["open"]) or match["sign"] in tuple(NEGATIVE_SIGNS)
    return Money(int(value.quantize(Decimal(1), rounding=ROUND_HALF_UP)), negative, direction)


def column_amount(text: str, vocab: Vocabulary) -> int | None:
    """Paise in a debit or credit cell; None for "-", blank or zero (the column is unused)."""
    money = parse_money(text, vocab)
    return money.paise if money and money.paise else None


def balance_paise(text: str, vocab: Vocabulary) -> int | None:
    """A balance, signed: a minus sign, parentheses or a Dr/D suffix make it negative."""
    money = parse_money(text, vocab)
    if money is None:
        return None
    return -money.paise if money.negative or money.direction == "debit" else money.paise


FOREIGN = re.compile(r"^(?:([A-Za-z]{3})\s*)?(\d[\d,]*\.\d+)$")


def parse_foreign(text: str) -> tuple[str, str | None] | None:
    """("25.00", "USD") from "USD 25.00": the amount without grouping, and its currency if printed."""
    match = FOREIGN.match(text.strip())
    if not match:
        return None
    return match.group(2).replace(",", ""), match.group(1).upper() if match.group(1) else None


def is_foreign(text: str) -> bool:
    return parse_foreign(text) is not None and any(c.isalpha() for c in text)
