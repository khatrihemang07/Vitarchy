"""The text of a page or sheet as positioned cells, for finding labels and the values next to them."""

from __future__ import annotations

from dataclasses import dataclass

from ..read.lines import Line
from ..read.sheet import Grid

FACT_GAP = 10.0  # words closer than this are one label ("Total Amount Due")
PAGE_STEP = 100_000.0  # rows of different pages are never "near" each other


@dataclass(frozen=True)
class FactCell:
    text: str
    x0: float
    x1: float
    row: float  # y in points (plus a page offset) for a page; the row number for a sheet
    page: int = 0


@dataclass
class Surface:
    cells: list[FactCell]
    below: float  # how far under a label its value may sit
    row_tolerance: float  # cells this close in `row` are on the same row
    is_sheet: bool = False

    def window(self, page: int, y0: float, y1: float) -> list[FactCell]:
        return [c for c in self.cells if c.page == page and y0 <= c.row - page * PAGE_STEP < y1]


def pdf_surface(lines: list[Line], below: float = 40.0) -> Surface:
    cells = [FactCell(c.text, c.x0, c.x1, line.page * PAGE_STEP + line.y, line.page)
             for line in lines for c in line.chunks(FACT_GAP)]
    return Surface(sorted(cells, key=lambda c: (c.row, c.x0)), below, 2.5)


def sheet_surface(grid: Grid, skip: range = range(0)) -> Surface:
    """Cells of a sheet; the rows in `skip` (the table's own header) are left out so a column
    label such as "Closing Balance" is never read as a statement figure."""
    cells = [FactCell(text.strip(), col, col, row)
             for row, cells_ in enumerate(grid) if row not in skip
             for col, text in enumerate(cells_) if text.strip()]
    return Surface(cells, below=2.0, row_tolerance=0.1, is_sheet=True)
