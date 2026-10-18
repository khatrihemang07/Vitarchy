"""Table regions per page: header (or carried layout) -> body lines up to a stop marker."""

from __future__ import annotations

from dataclasses import dataclass

from ..read.lines import Line, group_lines
from ..read.pdf_words import Page, Word
from ..segment.sections import Heading, find_heading
from ..vocabulary import Vocabulary
from .header import Header, find_header
from .layout import Layout
from .span import clip, probe, table_span
from .stops import END, PAGE_TOTAL, region_end

TOP, BOTTOM = float("-inf"), float("inf")


@dataclass
class Region:
    page: int
    layout: Layout
    lines: list[Line]  # span-filtered body lines, header and stop line excluded
    stop: str | None  # why it ended: "end", "page_total" or None (the page ended)
    heading: Heading | None
    open: bool = True  # the table may go on over the page: no closing marker was met
    carried: bool = False  # the layout came from an earlier page's header


def _between(words: list[Word], top: float, bottom: float) -> list[Word]:
    return [w for w in words if top < (w.y0 + w.y1) / 2 < bottom]


def body_region(page: Page, number: int, layout: Layout, top: float, bottom: float, vocab: Vocabulary,
                heading: Heading | None, carried: bool) -> Region:
    """The span-filtered lines between `top` and `bottom`, cut at the first stop marker."""
    words = _between(page.words, top, bottom)
    span = table_span(group_lines(probe(words, layout), number), layout, vocab)
    lines = group_lines(clip(words, span), number)
    end, kind = region_end(lines, 0, vocab)
    # A page-total row ends the page, not the table, unless a closing marker follows it.
    still_open = kind is None or (kind == PAGE_TOTAL and region_end(lines, end + 1, vocab)[1] != END)
    return Region(number, layout, lines[:end], kind, heading, still_open, carried)


def _header_regions(page: Page, number: int, lines: list[Line], vocab: Vocabulary) -> list[Region]:
    regions: list[Region] = []
    header: Header | None = find_header(lines, vocab)
    boundary = TOP
    while header:
        following = find_header(lines, vocab, header.last + 1)
        bottom = lines[following.first].y - 0.1 if following else BOTTOM
        above = [line for line in lines[:header.first] if line.y > boundary]
        region = body_region(page, number, header.layout, header.bottom + 0.1, bottom, vocab,
                             find_heading(above, vocab), carried=False)
        regions.append(region)
        boundary = bottom if following else BOTTOM
        header = following
    return regions


def locate_regions(pages: list[Page], vocab: Vocabulary) -> list[Region]:
    """Every table region of the document in order; a table's layout carries over pages without a header."""
    regions: list[Region] = []
    carried: Layout | None = None
    for number, page in enumerate(pages):
        lines = group_lines(page.words, number)
        found = _header_regions(page, number, lines, vocab)
        if not found and carried is not None:
            found = [body_region(page, number, carried, TOP, BOTTOM, vocab, None, carried=True)]
        if found:
            carried = found[-1].layout if found[-1].open else None
        regions.extend(found)
    return regions


HEADERLESS_LEAD = 45.0  # how far above its first row a headerless table's text may start


def headerless_regions(pages: list[Page], layout: Layout, vocab: Vocabulary) -> list[Region]:
    """Regions of a table with no header text: each page from just above its first dated row."""
    from ..segment.anchors import anchor_cells

    regions = []
    for number, page in enumerate(pages):
        lines = group_lines(probe(page.words, layout), number)
        firsts = [line.y for line in lines if anchor_cells(line, layout, vocab)]
        if firsts:
            regions.append(body_region(page, number, layout, firsts[0] - HEADERLESS_LEAD, BOTTOM,
                                       vocab, None, carried=False))
    return regions
