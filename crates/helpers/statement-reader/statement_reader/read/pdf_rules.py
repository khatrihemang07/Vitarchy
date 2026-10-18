"""A page's ruling lines (table borders), read from its vector drawings."""

from __future__ import annotations

from dataclasses import dataclass

import pymupdf

MIN_RULE_LENGTH = 3.0  # shorter strokes are glyph outlines, not table rules
RULE_TOLERANCE = 0.6  # a stroke this close to level counts as horizontal or vertical
THIN_FILL = 1.5  # a filled rectangle this thin is a ruling line drawn as a fill


@dataclass(frozen=True)
class Rule:
    """A horizontal or vertical line segment (x0 <= x1, y0 <= y1)."""

    x0: float
    y0: float
    x1: float
    y1: float

    @property
    def horizontal(self) -> bool:
        return (self.y1 - self.y0) <= (self.x1 - self.x0)


def page_rules(page: pymupdf.Page) -> list[Rule]:
    """The ruling lines of a page: strokes and thin filled rectangles that are level or upright."""
    rules: list[Rule] = []
    for path in page.get_drawings():
        stroked = path.get("type") in ("s", "fs")
        for item in path["items"]:
            if item[0] == "l":
                rules.extend(_segment(item[1], item[2]))
            elif item[0] == "re":
                rules.extend(_rectangle(item[1], stroked))
    return rules


def _segment(p1: pymupdf.Point, p2: pymupdf.Point) -> list[Rule]:
    x0, x1 = sorted((p1.x, p2.x))
    y0, y1 = sorted((p1.y, p2.y))
    level = y1 - y0 <= RULE_TOLERANCE and x1 - x0 >= MIN_RULE_LENGTH
    upright = x1 - x0 <= RULE_TOLERANCE and y1 - y0 >= MIN_RULE_LENGTH
    return [Rule(x0, y0, x1, y1)] if level or upright else []


def _rectangle(rect: pymupdf.Rect, stroked: bool) -> list[Rule]:
    x0, y0, x1, y1 = rect.x0, rect.y0, rect.x1, rect.y1
    if stroked:
        edges = [(x0, y0, x1, y0), (x0, y1, x1, y1), (x0, y0, x0, y1), (x1, y0, x1, y1)]
        return [r for e in edges for r in _segment(pymupdf.Point(e[0], e[1]), pymupdf.Point(e[2], e[3]))]
    if y1 - y0 <= THIN_FILL and x1 - x0 >= MIN_RULE_LENGTH:
        return [Rule(x0, y0, x1, y1)]
    if x1 - x0 <= THIN_FILL and y1 - y0 >= MIN_RULE_LENGTH:
        return [Rule(x0, y0, x1, y1)]
    return []
