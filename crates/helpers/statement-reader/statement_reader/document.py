"""An opened, readable file: pages for a PDF, a grid for a spreadsheet or CSV."""

from __future__ import annotations

from dataclasses import dataclass

from .read.pdf_words import PageReader
from .read.sheet import Grid


@dataclass
class Document:
    format: str  # pdf | csv | xls | xlsx
    pages: PageReader | None = None  # set for pdf
    grid: Grid | None = None  # set for csv, xls, xlsx
