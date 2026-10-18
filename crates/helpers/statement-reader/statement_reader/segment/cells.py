"""Puts the chunks of one printed line into the columns of a table layout."""

from __future__ import annotations

from ..locate.layout import NUMERIC_FIELDS, Column, Layout
from ..normalize.amounts import is_foreign, parse_money
from ..normalize.dates import date_prefix_words, parse_any
from ..read.lines import Chunk, split_chunks
from ..vocabulary import Vocabulary

MIN_OVERLAP = 0.5  # share of the chunk (or narrower side) that must lie in a column's span
SLACK = 6.0
SUFFIX_JOIN_GAP = 30.0


def _overlap(chunk: Chunk, x0: float, x1: float) -> float:
    return min(chunk.x1, x1) - max(chunk.x0, x0)


def _is_money(text: str, vocab: Vocabulary) -> bool:
    return text.strip() in ("-", "--") or parse_money(text, vocab) is not None or is_foreign(text)


def join_suffixes(chunks: list[Chunk], vocab: Vocabulary) -> list[Chunk]:
    """A lone Cr/Dr/C/D chunk right after an amount becomes part of that amount."""
    suffixes = {*vocab.directions["credit_suffixes"], *vocab.directions["debit_suffixes"]}
    joined: list[Chunk] = []
    for chunk in chunks:
        previous = joined[-1] if joined else None
        if (previous and chunk.text.lower() in suffixes and parse_money(previous.text, vocab)
                and chunk.x0 - previous.x1 <= SUFFIX_JOIN_GAP):
            joined[-1] = Chunk(f"{previous.text} {chunk.text}", previous.x0, chunk.x1, previous.bold)
        else:
            joined.append(chunk)
    return joined


def split_leading_date(chunks: list[Chunk], vocab: Vocabulary) -> list[Chunk]:
    """When a date is printed so close to the text after it that they form one chunk, split them."""
    if not chunks:
        return chunks
    first = chunks[0]
    words = first.text.split()
    count = date_prefix_words(words, vocab)
    if count == 0 or count == len(words):
        return chunks
    date_text, rest = " ".join(words[:count]), " ".join(words[count:])
    cut = first.x0 + (first.x1 - first.x0) * len(date_text) / len(first.text)
    return [Chunk(date_text, first.x0, cut, first.bold), Chunk(rest, cut, first.x1, first.bold), *chunks[1:]]


def split_at_columns(chunks: list[Chunk], layout: Layout, vocab: Vocabulary) -> list[Chunk]:
    """Cells printed so close together that they form one chunk are cut where a text column starts."""
    starts = sorted(c.x0 for c in layout.columns
                    if c.field not in NUMERIC_FIELDS and c.field not in ("ignored", "reference"))
    cut: list[Chunk] = []
    for chunk in chunks:
        if len(chunk.words) < 2 or _is_money(chunk.text, vocab) or parse_any(chunk.text, vocab):
            cut.append(chunk)
            continue
        pieces: list[list] = [[chunk.words[0]]]
        for word in chunk.words[1:]:
            previous = pieces[-1][-1]
            crossed = any(previous.x0 < start - SLACK / 2 <= word.x0 for start in starts)
            pieces.append([word]) if crossed else pieces[-1].append(word)
        cut.extend(split_chunks(sum(pieces, []), gap=1e9) if len(pieces) == 1 else
                   [split_chunks(p, gap=1e9)[0] for p in pieces])
    return cut


def _share(chunk: Chunk, column: Column) -> float:
    """Share of the chunk that lies inside the column's header span."""
    return _overlap(chunk, column.x0, column.x1) / max(chunk.x1 - chunk.x0, 1e-6)


def _money_column(chunk: Chunk, layout: Layout) -> Column | None:
    """A figure sits under its header, or, when right-aligned under a left-aligned label, right of
    it: the last figure column that starts before the figure ends."""
    over = max(layout.columns, key=lambda c: _share(chunk, c))
    if _share(chunk, over) >= MIN_OVERLAP:
        return over
    figures = [c for c in layout.columns if c.field in NUMERIC_FIELDS and c.x0 - SLACK <= chunk.x1]
    return figures[-1] if figures else None


def _text_column(chunk: Chunk, layout: Layout) -> Column | None:
    """A text cell starts where its column starts: the last text column that begins before it."""
    texts = [c for c in layout.columns if c.field not in NUMERIC_FIELDS and c.field != "ignored"]
    best = max(texts, key=lambda c: _overlap(chunk, c.x0, c.x1) / max(min(chunk.x1 - chunk.x0,
                                                                         c.x1 - c.x0), 1e-6), default=None)
    narrower = min(chunk.x1 - chunk.x0, best.x1 - best.x0) if best else 0
    if best and _overlap(chunk, best.x0, best.x1) >= MIN_OVERLAP * narrower:
        return best
    started = [c for c in texts if c.x0 - SLACK <= chunk.x0]
    if not started:
        return None
    column = started[-1]
    if chunk.x0 > column.x1 + SLACK:  # text in the gap right of a narrow label: the row's description
        return next((c for c in texts if c.field == "description"), column)
    return column


def assign(chunks: list[Chunk], layout: Layout, vocab: Vocabulary) -> dict[str, list[Chunk]]:
    """field -> chunks, left to right. Chunks that fit no other column are the description."""
    placed: dict[str, list[Chunk]] = {}
    for chunk in join_suffixes(chunks, vocab):
        money = _is_money(chunk.text, vocab)
        column = (_money_column(chunk, layout) if money else None) or _text_column(chunk, layout)
        field = column.field if column else "description"
        placed.setdefault(field, []).append(chunk)
    return placed
