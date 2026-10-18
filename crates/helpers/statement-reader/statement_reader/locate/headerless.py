"""Columns inferred from value types, for tables whose header is drawn as vector paths (no text).

Row-like lines (a date first, then at least two real amounts) vote for the date column, the
right-aligned money columns and the long text column. The last money column is the balance. When
two money columns remain their order (debit/credit) is unknown, so both layouts are offered and
the engine keeps the one whose balance chain holds.
"""

from __future__ import annotations

from ..normalize.amounts import parse_money
from ..normalize.dates import parse_any
from ..read.lines import Chunk, Line
from ..vocabulary import Vocabulary
from .layout import Column, Layout

MIN_ROWS = 2
EDGE_TOLERANCE = 8.0  # right edges within this many points are one money column
MIN_NUMERIC_SHARE = 0.5  # a second date column must appear on this share of rows
BLANKS = ("-", "--")


def _row_chunks(line: Line, vocab: Vocabulary) -> list[Chunk] | None:
    chunks = line.chunks(6.0)
    if not chunks or not parse_any(chunks[0].text, vocab):
        return None
    real = [c for c in chunks[1:] if parse_money(c.text, vocab) and c.text.strip() not in BLANKS]
    return chunks if len(real) >= 2 else None


def _clusters(chunks: list[Chunk]) -> list[list[Chunk]]:
    groups: list[list[Chunk]] = []
    for chunk in sorted(chunks, key=lambda c: c.x1):
        if groups and chunk.x1 - groups[-1][-1].x1 <= EDGE_TOLERANCE:
            groups[-1].append(chunk)
        else:
            groups.append([chunk])
    return groups


def _span(group: list[Chunk]) -> tuple[float, float]:
    return min(c.x0 for c in group), max(c.x1 for c in group)


def _build(dates: list[Column], money: list[Column], marks: list[Column], debit_first: bool) -> Layout:
    names = ["debit", "credit"] if debit_first else ["credit", "debit"]
    fields = ["amount"] if len(money) == 2 else [*names] if len(money) == 3 else []
    right = min(c.x0 for c in [*money, *marks])
    columns = [*dates, *marks, Column("description", dates[-1].x1 + 1, right - 1)]
    for column, field in zip(money, [*fields, "balance"], strict=True):
        columns.append(Column(field, column.x0, column.x1))
    return Layout(columns)


def infer_layouts(lines: list[Line], vocab: Vocabulary) -> list[Layout]:
    """Candidate layouts (best guess first), or [] when the lines do not look like a ledger."""
    rows = [chunks for line in lines if (chunks := _row_chunks(line, vocab))]
    if len(rows) < MIN_ROWS:
        return []
    firsts = [r[0] for r in rows]
    dates = [Column("date", min(c.x0 for c in firsts), max(c.x1 for c in firsts))]
    second = [r[1] for r in rows if parse_any(r[1].text, vocab)]
    if len(second) >= len(rows) * MIN_NUMERIC_SHARE:
        dates.append(Column("value_date", min(c.x0 for c in second), max(c.x1 for c in second)))
    pool = [c for r in rows for c in r[1:]
            if (parse_money(c.text, vocab) or c.text.strip() in BLANKS) and c.x0 > dates[-1].x1]
    money, marks = [], []
    for group in _clusters(pool):
        real = [c for c in group if c.text.strip() not in BLANKS]
        low, high = _span(group)
        if real:
            money.append(Column("money", low, high))
        else:
            marks.append(Column("ignored", low, high))
    if len(money) not in (2, 3):
        return []
    layouts = [_build(dates, money, marks, debit_first) for debit_first in (True, False)]
    return layouts if len(money) == 3 else layouts[:1]
