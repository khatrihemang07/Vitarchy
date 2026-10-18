"""Splits a bill's rows into one section per card: each card is its own Account."""

from __future__ import annotations

from dataclasses import dataclass

from .rows import RawRow


@dataclass
class CardSection:
    mask: str | None
    rows: list[RawRow]


def split_by_card(rows: list[RawRow], bill_mask: str | None) -> list[CardSection]:
    """One section per card, in printed order. Rows before the first card marker belong to
    the bill-level number when one is printed, else to the first card's section."""
    first_marker = next((r.card for r in rows if r.card), None)
    leading = bill_mask or first_marker
    sections: dict[str | None, list[RawRow]] = {}
    for row in rows:
        sections.setdefault(row.card or leading, []).append(row)
    return [CardSection(mask, part) for mask, part in sections.items()]
