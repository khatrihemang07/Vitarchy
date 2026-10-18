"""Finds the header row of a spreadsheet: the same vocabulary test as a PDF header, by column."""

from __future__ import annotations

from dataclasses import dataclass

from ..normalize.amounts import parse_money
from ..normalize.dates import parse_any
from ..read.sheet import Grid
from ..vocabulary import Vocabulary
from .header import MIN_FIELDS, is_valid_layout, resolve_field
from .layout import Column, Layout


@dataclass
class GridHeader:
    first: int
    last: int
    layout: Layout  # Column.x0 == Column.x1 == the column index


def _layout(rows: list[list[str]], vocab: Vocabulary) -> Layout:
    columns = []
    for index in range(len(rows[0])):
        texts = [" ".join(row[index].split()) for row in rows if row[index].strip()]
        field = resolve_field(texts, vocab) if texts else None
        if field:
            columns.append(Column(field, index, index))
    return Layout(columns)


def _label_row(row: list[str], vocab: Vocabulary) -> bool:
    cells = [c for c in row if c.strip()]
    return bool(cells) and not any(parse_any(c, vocab) or parse_money(c, vocab) for c in cells)


def find_grid_header(grid: Grid, vocab: Vocabulary) -> GridHeader | None:
    """The first row (or row pair, for labels split over two rows) that reads as a header."""
    for i, row in enumerate(grid):
        if sum(1 for c in row if c.strip()) < MIN_FIELDS - 1:
            continue
        single = _layout([row], vocab)
        nxt = grid[i + 1] if i + 1 < len(grid) else None
        if nxt and len(single.fields) >= 2 and _label_row(nxt, vocab):
            pair = _layout([row, nxt], vocab)
            if is_valid_layout(pair) and len(pair.fields) > len(single.fields):
                return GridHeader(i, i + 1, pair)
        if is_valid_layout(single):
            return GridHeader(i, i, single)
    return None
