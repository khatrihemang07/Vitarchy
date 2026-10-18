"""Rows of a spreadsheet table: one per dated row; description-only rows continue the row above."""

from __future__ import annotations

from ..locate.layout import DATE_FIELDS, Layout
from ..normalize.dates import parse_any
from ..read.sheet import Grid
from ..vocabulary import Vocabulary
from .keep import kept_text
from .rows import RawRow, pseudo_kind
from .wrap import cell_text, join_description


def _cells(row: list[str], layout: Layout) -> dict[str, str]:
    cells: dict[str, str] = {}
    for column in layout.columns:
        cells.setdefault(column.field, row[int(column.x0)].strip())
    return cells


def grid_rows(grid: Grid, first: int, layout: Layout, vocab: Vocabulary) -> list[RawRow]:
    """Rows from index `first` until the date cell stops parsing. Blank rows are skipped; a row
    with only its description filled (a remark that spilled over) joins the row above."""
    rows: list[RawRow] = []
    for row in grid[first:]:
        if not any(c.strip() for c in row):
            continue
        cells = _cells(row, layout)
        if any(parse_any(cells.get(f, ""), vocab) for f in DATE_FIELDS):
            description = join_description(
                [kept_text(f, line, vocab) for f in ("type", "reference")
                 for line in cells.get(f, "").splitlines()] + [cell_text(cells.get("description", ""))])
            cells.pop("description", None)
            rows.append(RawRow(cells, description, 0, pseudo_kind(description, vocab)))
            continue
        filled = [c for c in row if c.strip()]
        description_cell = _cells(row, layout).get("description", "")
        if rows and len(filled) == 1 and filled[0].strip() == description_cell:
            rows[-1].description = join_description([rows[-1].description, *description_cell.splitlines()])
        else:
            break
    return rows
