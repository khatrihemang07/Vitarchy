"""Finds a table header: a line, or two adjacent lines merged, naming date, money and description columns.

Cells are whole chunks of text matched against the vocabulary, longest synonym first, so
"Credit/debit instrument" is an instrument column and never a credit or a debit.
"""

from __future__ import annotations

from dataclasses import dataclass

from ..normalize.amounts import parse_money
from ..normalize.dates import parse_any
from ..read.lines import Chunk, Line
from ..vocabulary import Vocabulary
from .layout import Column, Layout, has_date_and_money

HEADER_GAP = 5.0  # points between header labels; narrower gaps join words of one label
MAX_LINE_DISTANCE = 14.0  # a second header line sits at most this far below the first
MIN_FIELDS = 3
MAX_LABEL_LINES = 3
OVERLAP_SLACK = 3.0


@dataclass
class Header:
    first: int  # index of the first header line
    last: int  # index of the last header line
    layout: Layout
    bottom: float  # y of the last header line


@dataclass
class _Cell:
    texts: list[str]
    x0: float
    x1: float


def resolve_field(texts: list[str], vocab: Vocabulary) -> str | None:
    """Field for one header cell made of one or two stacked labels (upper first)."""
    joined = " ".join(texts)
    candidates = [joined, *reversed(texts)] if len(texts) > 1 else [joined]
    for text in candidates:
        if field := vocab.field_of(text, allow_prefix=False):
            return field
    for text in [joined, *texts]:
        if field := vocab.field_of(text):
            return field
    return None


def _overlaps(cell: _Cell, chunk: Chunk) -> bool:
    return chunk.x0 <= cell.x1 + OVERLAP_SLACK and chunk.x1 >= cell.x0 - OVERLAP_SLACK


def _cells(lines: list[Line]) -> list[_Cell]:
    cells: list[_Cell] = []
    for line in lines:
        for chunk in line.chunks(HEADER_GAP):
            target = next((c for c in cells if _overlaps(c, chunk)), None) if cells else None
            if target is None:
                cells.append(_Cell([chunk.text], chunk.x0, chunk.x1))
            else:
                target.texts.append(chunk.text)
                target.x0, target.x1 = min(target.x0, chunk.x0), max(target.x1, chunk.x1)
    return cells


def _layout(lines: list[Line], vocab: Vocabulary, with_unknown: bool = False) -> Layout:
    """Recognised cells as columns. With `with_unknown`, other cells (and columns the vocabulary
    only marks as ignored) lying inside the table's span become ignored columns, so their body
    text stays out of the description; those outside the span are not part of the table."""
    cells = [(resolve_field(c.texts, vocab), c) for c in _cells(lines)]
    core = [Column(field, c.x0, c.x1) for field, c in cells if field and field != "ignored"]
    if not with_unknown or not core:
        return Layout(core)
    low, high = min(c.x0 for c in core), max(c.x1 for c in core)
    extra = [Column("ignored", c.x0, c.x1) for field, c in cells
             if (not field or field == "ignored") and low <= c.x0 and c.x1 <= high]
    return Layout(core + extra)


def _header_like(line: Line, vocab: Vocabulary) -> bool:
    return not any(parse_any(c.text, vocab) or parse_money(c.text, vocab) for c in line.chunks(HEADER_GAP))


def is_valid_layout(layout: Layout) -> bool:
    """Three or more concepts including a date, money and the text of each row."""
    return (len(layout.fields) >= MIN_FIELDS and has_date_and_money(layout.fields)
            and "description" in layout.fields)


def _bottom(line: Line) -> float:
    """Lowest word centre of a header line (words on one line can differ by a point or two)."""
    return max((w.y0 + w.y1) / 2 for w in line.words)


def _window(lines: list[Line], i: int, vocab: Vocabulary) -> Header | None:
    """The best header starting at line `i`: up to MAX_LABEL_LINES stacked lines, most concepts wins."""
    best: Header | None = None
    best_score = 0
    for count in range(1, MAX_LABEL_LINES + 1):
        taken = lines[i:i + count]
        if len(taken) < count or (count > 1 and (taken[-1].y - taken[-2].y > MAX_LINE_DISTANCE
                                                  or not _header_like(taken[-1], vocab))):
            break
        layout = _layout(taken, vocab)
        if is_valid_layout(layout) and len(layout.fields) > best_score:
            best_score = len(layout.fields)
            best = Header(i, i + count - 1, _layout(taken, vocab, True), _bottom(taken[-1]))
        elif count == 1 and not layout.columns:
            break
    return best


def find_header(lines: list[Line], vocab: Vocabulary, start: int = 0) -> Header | None:
    """The first header at or after line `start`; labels may be stacked over up to three lines."""
    for i in range(start, len(lines)):
        if found := _window(lines, i, vocab):
            return found
    return None
