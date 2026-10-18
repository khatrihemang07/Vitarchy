"""Where a table region ends on a page: the first stop marker from the vocabulary."""

from __future__ import annotations

from ..read.lines import Line
from ..vocabulary import Vocabulary

END = "end"  # the table is over (summary, legend, terms)
PAGE_TOTAL = "page_total"  # only this page's rows end here; the table may go on next page


def stop_kind(line: Line, vocab: Vocabulary) -> str | None:
    text = line.text.strip().lower()
    if text.startswith(vocab.rows["page_total_markers"]):
        return PAGE_TOTAL
    return END if vocab.is_stop_line(line.text) else None


def region_end(lines: list[Line], start: int, vocab: Vocabulary) -> tuple[int, str | None]:
    """(index of the stopping line or len(lines), the kind of stop or None when the page just ends)."""
    for index in range(start, len(lines)):
        if kind := stop_kind(lines[index], vocab):
            return index, kind
    return len(lines), None
