"""Which way a row moved money, and by how much. Priority: separate debit/credit columns, a type
word, an amount suffix, the amount's sign, then the change in the running balance. A plain amount
with none of those is a debit (a card charge)."""

from __future__ import annotations

import re
from dataclasses import dataclass

from ..vocabulary import Vocabulary
from .amounts import Money


@dataclass
class Figures:
    """What one row printed, already parsed."""

    debit: int | None = None
    credit: int | None = None
    amount: Money | None = None
    type_text: str | None = None
    balance: int | None = None
    previous_balance: int | None = None


def _type_direction(text: str | None, vocab: Vocabulary) -> str | None:
    tokens = set(re.split(r"[^a-z]+", (text or "").lower()))
    debit = tokens & set(vocab.directions["debit_words"])
    credit = tokens & set(vocab.directions["credit_words"])
    if debit and not credit:
        return "debit"
    if credit and not debit:
        return "credit"
    return None


def _balance_direction(figures: Figures, inverted: bool) -> str | None:
    if figures.balance is None or figures.previous_balance is None:
        return None
    delta = figures.balance - figures.previous_balance
    if delta == 0:
        return None
    return "credit" if (delta > 0) != inverted else "debit"


def decide(figures: Figures, vocab: Vocabulary, negative_means: str = "debit",
           use_balance: bool = False, inverted: bool = False) -> tuple[str, int] | None:
    """(direction, paise) for a row, or None when the row has no amount at all."""
    if figures.debit and not figures.credit:
        return "debit", figures.debit
    if figures.credit and not figures.debit:
        return "credit", figures.credit
    amount = figures.amount
    if figures.debit and figures.credit:
        net = figures.debit - figures.credit
        return ("debit", net) if net > 0 else ("credit", -net)
    if amount is None or amount.paise == 0:
        return None
    direction = _type_direction(figures.type_text, vocab) or amount.direction
    if direction is None and amount.negative:
        direction = negative_means
    if direction is None and use_balance:
        direction = _balance_direction(figures, inverted)
    return direction or "debit", amount.paise
