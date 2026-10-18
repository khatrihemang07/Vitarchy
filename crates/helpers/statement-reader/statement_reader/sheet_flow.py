"""Spreadsheet or CSV -> header row -> rows -> Statement, or Payment records for a payment app."""

from __future__ import annotations

from .contract import Diagnostics, Result
from .locate.grid_header import find_grid_header
from .metadata.surface import sheet_surface
from .normalize.payments import build_payment_records
from .normalize.rows import NormaliseError
from .read.sheet import Grid
from .segment.grid_rows import grid_rows
from .statements import make_statements, statement_facts
from .vocabulary import load_vocabulary


def read_sheet(grid: Grid) -> Result:
    vocab = load_vocabulary()
    header = find_grid_header(grid, vocab)
    if header is None:
        failure = Diagnostics("locate", "no_header_or_columns", rows=len(grid))
        return Result(outcome="no_table", diagnostics=failure)
    rows = grid_rows(grid, header.last + 1, header.layout, vocab)
    if not rows:
        return Result(outcome="no_table", diagnostics=Diagnostics("segment", "no_rows", rows=len(grid)))
    diagnostics = Diagnostics(rows=len(rows))
    try:
        if set(vocab.payment_history["header_fields"]) <= header.layout.fields:
            records = build_payment_records([r for r in rows if r.pseudo is None], vocab)
            return Result(outcome="read", kind="payment_history", payment_records=records,
                          diagnostics=diagnostics)
        surface = sheet_surface(grid, range(header.first, header.last + 1))
        statements = make_statements(rows, statement_facts(surface, None, vocab), surface, vocab)
    except NormaliseError as error:
        return Result(outcome="no_table", diagnostics=Diagnostics("normalize", error.code, rows=len(rows)))
    return Result(outcome="read", kind="statement", statements=statements, diagnostics=diagnostics)
