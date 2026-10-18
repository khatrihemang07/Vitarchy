"""The horizontal extent of a table: its header cells and its body words, so side panels fall outside."""

from __future__ import annotations

from ..read.lines import CHUNK_GAP, Line, group_lines
from ..read.pdf_words import Word
from ..segment.anchors import anchor_cells
from ..vocabulary import Vocabulary
from .layout import Layout

PROBE = 25.0  # how far left of the header the first look for row cells reaches
PAD = 2.0


def runs(words: list[Word]) -> list[list[Word]]:
    """Runs of words on one line with small gaps between them: one printed text each."""
    found: list[list[Word]] = []
    for line in group_lines(words):
        for chunk_words in _split(line.words):
            found.append(chunk_words)
    return found


def _split(words: list[Word]) -> list[list[Word]]:
    groups: list[list[Word]] = []
    for word in words:
        if groups and word.x0 - groups[-1][-1].x1 <= CHUNK_GAP:
            groups[-1].append(word)
        else:
            groups.append([word])
    return groups


def probe(words: list[Word], layout: Layout) -> list[Word]:
    """Whole texts from just left of the header to the page edge: enough to find the rows."""
    return [w for run in runs(words) if run[0].x0 >= layout.x_min - PROBE for w in run]


def table_span(lines: list[Line], layout: Layout, vocab: Vocabulary) -> tuple[float, float]:
    """(left, right): the header's extent, widened to where row cells (not description text) reach."""
    left, right = layout.x_min, layout.x_max
    for line in lines:
        placed = anchor_cells(line, layout, vocab)
        for field, chunks in (placed or {}).items():
            if field != "description":
                left = min(left, *(c.x0 for c in chunks))
                right = max(right, *(c.x1 for c in chunks))
    return left, right


def clip(words: list[Word], span: tuple[float, float]) -> list[Word]:
    """Whole texts inside the span: a text that starts in a side panel is dropped even if it
    runs into the table's columns."""
    return [w for run in runs(words) if run[0].x0 >= span[0] - PAD and run[-1].x1 <= span[1] + PAD
            for w in run]
