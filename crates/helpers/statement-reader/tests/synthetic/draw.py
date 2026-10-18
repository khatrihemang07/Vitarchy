"""Small pymupdf drawing helpers for the synthetic PDF layouts. Counts the visible words it draws."""

from __future__ import annotations

import pymupdf

A4 = (595, 842)


def text_width(s: str, size: float = 8, bold: bool = False) -> float:
    return pymupdf.get_text_length(s, fontname="hebo" if bold else "helv", fontsize=size)


def wrap_words(s: str, width: float, size: float = 8) -> list[str]:
    """Greedy word wrap (breaks only between words)."""
    lines, current = [], ""
    for word in s.split():
        trial = f"{current} {word}".strip()
        if current and text_width(trial, size) > width:
            lines.append(current)
            current = word
        else:
            current = trial
    return [*lines, current]


def hard_wrap(s: str, width: float, size: float = 8) -> list[str]:
    """Break as soon as a line reaches `width` points, even inside a token (printed reports do this)."""
    lines, current = [], ""
    for char in s:
        if current and text_width(current + char, size) > width:
            lines.append(current)
            current = ""
        current += char
    return [*lines, current]


class Pdf:
    def __init__(self) -> None:
        self.doc = pymupdf.open()
        self.words = 0
        self.page: pymupdf.Page

    def new_page(self) -> None:
        self.page = self.doc.new_page(width=A4[0], height=A4[1])

    def text(self, x: float, y: float, s: str, size: float = 8, bold: bool = False,
             right: bool = False, invisible: bool = False, rotate: int = 0,
             color: tuple = (0, 0, 0)) -> None:
        """Draw `s` with its baseline at y; x is the right edge when `right`."""
        if right:
            x -= text_width(s, size, bold)
        self.page.insert_text((x, y), s, fontsize=size, color=color, rotate=rotate,
                              fontname="hebo" if bold else "helv",
                              render_mode=3 if invisible else 0)
        if not invisible and rotate == 0:
            self.words += len(s.split())

    def hline(self, x0: float, x1: float, y: float, width: float = 0.5, color=(0.5, 0.5, 0.5)):
        self.page.draw_line((x0, y), (x1, y), color=color, width=width)

    def vline(self, x: float, y0: float, y1: float, width: float = 0.5, color=(0.5, 0.5, 0.5)):
        self.page.draw_line((x, y0), (x, y1), color=color, width=width)

    def rect(self, x0: float, y0: float, x1: float, y1: float, fill=None, stroke=None):
        self.page.draw_rect((x0, y0, x1, y1), color=stroke, fill=fill, width=0.5)

    def save(self, path, **options) -> None:
        # Fixed metadata keeps regenerated files byte-identical (no churn in git).
        stamp = "D:20260101000000Z"
        self.doc.set_metadata({"creationDate": stamp, "modDate": stamp, "producer": "synthetic",
                               "creator": "synthetic"})
        self.doc.save(path, no_new_id=True, **options)
        self.doc.close()
