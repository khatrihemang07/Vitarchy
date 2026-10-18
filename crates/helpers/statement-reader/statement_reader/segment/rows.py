"""Turns the lines of one table region into raw rows: date-anchored, narration attached by distance."""

from __future__ import annotations

import statistics
from dataclasses import dataclass, field

from ..locate.layout import Layout
from ..read.lines import Line
from ..vocabulary import Vocabulary
from .anchors import BODY_GAP, anchor_cells, line_cells
from .attach import attach_lines
from .keep import is_figures_only, line_edge, line_text
from .sections import card_marker, is_cardholder_divider
from .wrap import join_description

MIN_LIMIT, MAX_LIMIT, LIMIT_FACTOR = 12.0, 40.0, 1.5
OPENING, CLOSING = "opening", "closing"


@dataclass
class RawRow:
    cells: dict[str, str]  # field -> printed text, description excluded
    description: str
    page: int
    pseudo: str | None = None  # "opening" / "closing" balance rows are not Transactions
    card: str | None = None  # masked card of the section the row sits in


@dataclass
class Segmented:
    rows: list[RawRow] = field(default_factory=list)
    card: str | None = None  # the card section in force at the end of the region


FULL_SLACK = 12.0  # a line ending this close to the widest one was cut by the wrap width


def _wrap_width(edges: list[float]) -> float:
    """Where lines wrap: the 90th percentile of right edges, so a few stray wide lines (a footer
    sentence) do not move it."""
    ordered = sorted(e for e in edges if e)
    return ordered[int(0.9 * (len(ordered) - 1))] if ordered else 0.0


def _description(lines: list[Line], layout: Layout, vocab: Vocabulary, limit: float) -> str:
    """Lines joined as printed. A change of weight starts a new part (a bold first line is never
    the front half of the token the next line carries), so parts are joined with a space."""
    parts: list[tuple[list[str], list[bool]]] = []
    previous_bold: bool | None = None
    for line in lines:
        placed = line_cells(line, layout, vocab)
        if previous_bold is None or line.bold != previous_bold:
            parts.append(([], []))
        parts[-1][0].append(line_text(placed, vocab))
        parts[-1][1].append(line_edge(placed, vocab) >= limit - FULL_SLACK)
        previous_bold = line.bold
    return " ".join(t for t in (join_description(texts, full) for texts, full in parts) if t)


def pseudo_kind(description: str, vocab: Vocabulary) -> str | None:
    text = description.strip().lower()
    if text.startswith(vocab.rows["opening_markers"]):
        return OPENING
    return CLOSING if text.startswith(vocab.rows["closing_markers"]) else None


def _limit(lines: list[Line]) -> float:
    gaps = [b.y - a.y for a, b in zip(lines, lines[1:], strict=False) if b.y > a.y]
    typical = statistics.median(gaps) if gaps else MIN_LIMIT
    return min(MAX_LIMIT, max(MIN_LIMIT, LIMIT_FACTOR * typical))


def _is_furniture(line: Line, vocab: Vocabulary) -> bool:
    text = line.text.strip().lower()
    return text.startswith((*vocab.noise_startswith, *vocab.rows["opening_markers"])) \
        or is_figures_only(line.chunks(BODY_GAP), vocab)


def segment_region(lines: list[Line], layout: Layout, vocab: Vocabulary,
                   card: str | None = None) -> Segmented:
    """Raw rows of one region. Card markers and cardholder dividers split it into sections."""
    result = Segmented(card=card)
    section: list[Line] = []

    def flush(section_card: str | None) -> None:
        result.rows.extend(_section_rows(section, layout, vocab, section_card))
        section.clear()

    for line in lines:
        marker = card_marker(line, vocab)
        if marker or is_cardholder_divider(line, vocab):
            flush(result.card)
            result.card = marker or result.card
        else:
            section.append(line)
    flush(result.card)
    return result


def _section_rows(lines: list[Line], layout: Layout, vocab: Vocabulary,
                  card: str | None) -> list[RawRow]:
    kept = [(line, placed) for line in lines
            if (placed := anchor_cells(line, layout, vocab)) or not _is_furniture(line, vocab)]
    body = [line for line, _ in kept]
    anchors = {i: placed for i, (_, placed) in enumerate(kept) if placed}
    descriptions = {i: " ".join(c.text for c in placed.get("description", []))
                    for i, placed in anchors.items()}
    pseudo = {i for i, text in descriptions.items() if pseudo_kind(text, vocab)}
    limit = _wrap_width([line_edge(line_cells(line, layout, vocab), vocab) for line in body])
    members = {i: [i] for i in anchors}
    texty = {i for i, text in descriptions.items() if text.strip()}
    for index, owner in attach_lines(body, set(anchors), pseudo, _limit(body), texty).items():
        members[owner].append(index)
    rows = []
    for anchor, indices in members.items():
        description = _description([body[i] for i in sorted(indices)], layout, vocab, limit)
        cells = {f: " ".join(c.text for c in chunks)
                 for f, chunks in anchors[anchor].items() if f != "description"}
        rows.append(RawRow(cells, description, body[anchor].page, pseudo_kind(description, vocab), card))
    return rows
