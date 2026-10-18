"""A statement period pieced together from dated labels ("Your Opening Balance on 01-08-26")."""

from __future__ import annotations

from datetime import date

from ..normalize.dates import date_prefix_words, parse_any
from ..vocabulary import Vocabulary
from .parsers import period_value
from .surface import Surface


def date_after(surface: Surface, phrases: tuple[str, ...], vocab: Vocabulary) -> date | None:
    """The first date printed straight after one of `phrases` inside a cell."""
    for cell in surface.cells:
        low = cell.text.lower()
        for phrase in phrases:
            at = low.find(phrase)
            if at < 0:
                continue
            words = cell.text[at + len(phrase):].replace(":", " ").split()
            count = date_prefix_words(words, vocab)
            found = parse_any(" ".join(words[:count]), vocab) if count else None
            if found:
                return found[0]
    return None


def unlabelled_period(surface: Surface, vocab: Vocabulary) -> tuple[date | None, date | None] | None:
    """A cell that is nothing but a date range ("19 May, 2026 - 18 Jun, 2026"), with no label."""
    for cell in surface.cells:
        if (found := period_value(cell.text.strip(), vocab)) is not None:
            return found
    return None


def fill_period(period: tuple[date | None, date | None], surface: Surface,
                vocab: Vocabulary) -> tuple[date | None, date | None]:
    start = period[0] or date_after(surface, vocab.facts["period_start"], vocab)
    end = period[1] or date_after(surface, vocab.facts["period_end"], vocab)
    return start, end
