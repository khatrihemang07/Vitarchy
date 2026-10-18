"""Which printed lines are row anchors: a date in the date column plus an amount."""

from __future__ import annotations

from ..locate.layout import MONEY_FIELDS, Layout
from ..normalize.amounts import parse_money
from ..normalize.dates import parse_any
from ..read.lines import Chunk, Line
from ..vocabulary import Vocabulary
from .cells import assign, split_at_columns, split_leading_date

BODY_GAP = 6.0


def line_cells(line: Line, layout: Layout, vocab: Vocabulary) -> dict[str, list[Chunk]]:
    chunks = split_at_columns(split_leading_date(line.chunks(BODY_GAP), vocab), layout, vocab)
    return assign(chunks, layout, vocab)


def anchor_cells(line: Line, layout: Layout, vocab: Vocabulary) -> dict[str, list[Chunk]] | None:
    """The line's cells when it is a row anchor (date in the date column and an amount), else None."""
    date_column = layout.date_column
    placed = line_cells(line, layout, vocab)
    chunks = placed.get(date_column.field) if date_column else None
    if not chunks or not parse_any(chunks[0].text, vocab):
        return None
    if not any(parse_money(c.text, vocab) for f in MONEY_FIELDS for c in placed.get(f, [])):
        return None
    return placed
