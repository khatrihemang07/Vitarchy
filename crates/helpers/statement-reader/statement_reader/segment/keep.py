"""Text printed in a row's non-description columns must not vanish: it is kept in the description."""

from __future__ import annotations

import re

from ..normalize.amounts import parse_money
from ..read.lines import Chunk
from ..vocabulary import Vocabulary

KEEP_FIELDS = ("type", "reference", "description")  # columns whose text belongs to the row's text
BLANK_MARKS = ("", "-", "--")


def kept_text(field: str, text: str, vocab: Vocabulary) -> str:
    """`text` if it is part of the row's printed text, else "". A bare direction word in a type
    column (DEBIT, Cr) says which way money moved and is read by the direction stage instead."""
    text = " ".join(text.split())
    if field not in KEEP_FIELDS or text in BLANK_MARKS:
        return ""
    if field == "type":
        words = set(re.split(r"[^a-z]+", text.lower())) - {""}
        if words <= {*vocab.directions["debit_words"], *vocab.directions["credit_words"]}:
            return ""
    return text


def line_text(placed: dict[str, list[Chunk]], vocab: Vocabulary) -> str:
    """One printed line's kept text, left to right as printed."""
    chunks = sorted((c for f, cs in placed.items() for c in cs if kept_text(f, c.text, vocab)),
                    key=lambda c: c.x0)
    return " ".join(c.text for c in chunks)


def is_figures_only(chunks: list[Chunk], vocab: Vocabulary) -> bool:
    """A line of nothing but decimal amounts: a balance whose label was cut off, never narration."""
    return bool(chunks) and all("." in c.text and parse_money(c.text, vocab) for c in chunks)


def line_edge(placed: dict[str, list[Chunk]], vocab: Vocabulary) -> float:
    """Right edge of the line's kept text (0 when it has none)."""
    return max((c.x1 for f, cs in placed.items() for c in cs if kept_text(f, c.text, vocab)), default=0.0)
