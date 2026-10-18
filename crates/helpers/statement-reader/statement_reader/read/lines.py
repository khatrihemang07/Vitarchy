"""Words grouped into lines, and lines into chunks (runs of words with small gaps between them)."""

from __future__ import annotations

from dataclasses import dataclass, field

from .pdf_words import Word

LINE_TOLERANCE = 2.5  # words whose vertical centres differ by less than this share a line
CHUNK_GAP = 6.0  # a gap wider than this (points) separates two cells of a table row


@dataclass(frozen=True)
class Chunk:
    text: str
    x0: float
    x1: float
    bold: bool = False
    words: tuple[Word, ...] = field(default=(), compare=False, repr=False)

    @property
    def centre(self) -> float:
        return (self.x0 + self.x1) / 2


def split_chunks(words: list[Word], gap: float = CHUNK_GAP) -> list[Chunk]:
    """Left-to-right runs of words; a gap wider than `gap` starts a new chunk."""
    groups: list[list[Word]] = []
    for word in sorted(words, key=lambda w: w.x0):
        if groups and word.x0 - groups[-1][-1].x1 <= gap:
            groups[-1].append(word)
        else:
            groups.append([word])
    return [Chunk(" ".join(w.text for w in g), g[0].x0, max(w.x1 for w in g),
                  all(w.bold for w in g), tuple(g)) for g in groups]


@dataclass
class Line:
    y: float  # vertical centre
    words: list[Word]
    page: int = 0
    _chunks: dict[float, list[Chunk]] = field(default_factory=dict, repr=False)

    @property
    def text(self) -> str:
        return " ".join(w.text for w in self.words)

    @property
    def bold(self) -> bool:
        return all(w.bold for w in self.words)

    def chunks(self, gap: float = CHUNK_GAP) -> list[Chunk]:
        if gap not in self._chunks:
            self._chunks[gap] = split_chunks(self.words, gap)
        return self._chunks[gap]


def group_lines(words: list[Word], page: int = 0, tolerance: float = LINE_TOLERANCE) -> list[Line]:
    """Lines top to bottom, each with its words left to right."""
    lines: list[list[Word]] = []
    centres: list[float] = []
    for word in sorted(words, key=lambda w: ((w.y0 + w.y1) / 2, w.x0)):
        centre = (word.y0 + word.y1) / 2
        if lines and abs(centres[-1] / len(lines[-1]) - centre) <= tolerance:
            lines[-1].append(word)
            centres[-1] += centre
        else:
            lines.append([word])
            centres.append(centre)
    return [Line(total / len(ws), sorted(ws, key=lambda w: w.x0), page)
            for ws, total in zip(lines, centres, strict=True)]
