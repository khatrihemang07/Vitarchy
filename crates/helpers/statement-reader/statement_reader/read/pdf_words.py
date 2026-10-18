"""PageReader seam: a page as positioned words and ruling lines (pymupdf behind it)."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Protocol

import pymupdf

from ..errors import OpenError
from ..source import Source, read_bytes
from .pdf_rules import Rule, page_rules

HORIZONTAL = 0.99  # text direction x-component of unrotated, left-to-right text
BOLD_NAMES = ("bold", "black", "heavy")
NOT_BOLD_NAMES = ("semibold", "demibold")


@dataclass(frozen=True)
class Word:
    x0: float
    y0: float
    x1: float
    y1: float
    text: str
    bold: bool = False


@dataclass(frozen=True)
class Page:
    width: float
    height: float
    words: list[Word] = field(default_factory=list)
    rules: list[Rule] = field(default_factory=list)


class PageReader(Protocol):
    """Anything that can serve pages: pymupdf today, an OCR or layout reader later."""

    @property
    def page_count(self) -> int: ...

    def read_page(self, index: int) -> Page: ...

    def close(self) -> None: ...


def _is_type3(font: str) -> bool:
    return "-T3" in font or font.endswith("T3")


def is_bold(font: str, flags: int) -> bool:
    name = font.lower().replace("-", "").replace(" ", "")
    if any(token in name for token in NOT_BOLD_NAMES):
        return False
    return bool(flags & 16) or any(token in name for token in BOLD_NAMES)


class PyMuPdfReader:
    def __init__(self, source: Source) -> None:
        try:
            self._document = pymupdf.open(stream=read_bytes(source), filetype="pdf")
        except Exception as error:
            raise OpenError("corrupt_file") from error

    @property
    def page_count(self) -> int:
        return self._document.page_count

    def read_page(self, index: int) -> Page:
        page = self._document[index]
        return Page(page.rect.width, page.rect.height, _words(page), page_rules(page))

    def close(self) -> None:
        self._document.close()


def _words(page: pymupdf.Page) -> list[Word]:
    words: list[Word] = []
    seen: set[tuple] = set()
    for block in page.get_text("rawdict")["blocks"]:
        for line in block.get("lines", []):
            if line["dir"][0] < HORIZONTAL:
                continue  # rotated text, such as a page code on the margin
            for word in _line_words(line):
                key = (word.text, round(word.x0, 1), round(word.y0, 1), round(word.x1, 1))
                if word.text != "null" and key not in seen:  # drop `null` tokens and overlay copies
                    seen.add(key)
                    words.append(word)
    return words


def _line_words(line: dict) -> list[Word]:
    """Split a line's characters on whitespace; a span change does not end a word."""
    words: list[Word] = []
    chars: list[dict] = []
    bold = False

    def flush() -> None:
        if chars:
            text = "".join(c["c"] for c in chars)
            x0, y0 = min(c["bbox"][0] for c in chars), min(c["bbox"][1] for c in chars)
            x1, y1 = max(c["bbox"][2] for c in chars), max(c["bbox"][3] for c in chars)
            words.append(Word(x0, y0, x1, y1, text, bold))
            chars.clear()

    for span in line["spans"]:
        # Type3 glyphs report alpha 0 although they are drawn, so only other fonts count as invisible.
        if span.get("alpha", 255) == 0 and not _is_type3(span["font"]):
            flush()
            continue  # invisible text
        for char in span["chars"]:
            if char["c"].isspace():
                flush()
            else:
                if not chars:
                    bold = is_bold(span["font"], span["flags"])
                chars.append(char)
    flush()
    return words


def open_pdf(source: Source) -> PageReader:
    return PyMuPdfReader(source)
