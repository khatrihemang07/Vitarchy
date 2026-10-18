"""Finds the value printed next to a label: inside the same cell, to its right, or under it."""

from __future__ import annotations

from collections.abc import Callable

from .surface import FactCell, Surface

MAX_STACK = 3  # a label wrapped over this many lines is still matched
STACK_GAP = 12.0  # points between lines of one wrapped label
SLACK = 6.0  # horizontal give when lining a value up under a label
EXTRA_LABEL_TEXT = 36  # characters a label cell may hold beyond the phrase (dates, units)
MAX_RIGHT_CELLS = 4


def _collapse(text: str) -> str:
    return " ".join(text.split())


def _phrase_end(low: str, phrases: tuple[str, ...], contains: bool) -> int | None:
    for phrase in sorted(phrases, key=len, reverse=True):
        start = low.find(phrase) if contains else (0 if low.startswith(phrase) else -1)
        if start < 0:
            continue
        end = start + len(phrase)
        if (contains or len(low) <= end + EXTRA_LABEL_TEXT) and (end == len(low) or not low[end].isalpha()):
            return end
    return None


def _stack(surface: Surface, first: FactCell, wanted: tuple[str, ...]) -> list[FactCell]:
    """`first` plus the cells stacked under it that continue a wrapped label."""
    stack = [first]
    if surface.is_sheet or not any(p.startswith(_collapse(first.text).lower()) and len(first.text) >= 3
                                   for p in wanted):
        return stack
    for _ in range(MAX_STACK - 1):
        last = stack[-1]
        under = [c for c in surface.cells if 0 < c.row - last.row <= STACK_GAP
                 and any(ch.isalpha() for ch in c.text)
                 and c.x0 <= max(s.x1 for s in stack) and c.x1 >= min(s.x0 for s in stack)]
        if not under:
            break
        stack.append(min(under, key=lambda c: c.row))
    return stack


def _right(surface: Surface, stack: list[FactCell]) -> list[FactCell]:
    last = stack[-1]
    return sorted((c for c in surface.cells if abs(c.row - last.row) <= surface.row_tolerance
                   and c.x0 > last.x1 - (0 if surface.is_sheet else 1) and c is not last),
                  key=lambda c: c.x0)[:MAX_RIGHT_CELLS]


def _below(surface: Surface, stack: list[FactCell]) -> list[FactCell]:
    last = stack[-1]
    left, right = min(c.x0 for c in stack) - SLACK, max(c.x1 for c in stack) + SLACK
    if surface.is_sheet:
        left, right = left + SLACK - 0.5, right - SLACK + 0.5
    found = [c for c in surface.cells if surface.row_tolerance < c.row - last.row <= surface.below
             and c.x0 <= right and c.x1 >= left]
    return sorted(found, key=lambda c: (c.row, c.x0))


def _try[T](parse: Callable[[str], T | None], texts: list[str]) -> T | None:
    for count in range(1, len(texts) + 1):
        value = parse(" ".join(texts[:count]))
        if value is not None:
            return value
    return None


def find_value[T](surface: Surface, phrases: tuple[str, ...], parse: Callable[[str], T | None],
                  contains: bool = False) -> T | None:
    """The first value (reading order) that `parse` accepts next to a cell matching one of `phrases`."""
    for first in surface.cells:
        stack = _stack(surface, first, phrases)
        low = _collapse(" ".join(c.text for c in stack)).lower()
        own_low = _collapse(first.text).lower()
        end = _phrase_end(low, phrases, contains) or _phrase_end(own_low, phrases, contains)
        if end is None:
            continue
        own = _collapse(first.text)
        rest = own[_phrase_end(own.lower(), phrases, contains) or 0:] if len(stack) == 1 else ""
        candidates = [rest.rsplit(":", 1)[-1], rest]
        for text in candidates:
            if text.strip() and (value := parse(text.strip(" :"))) is not None:
                return value
        for value in (_try(parse, [c.text for c in _right(surface, stack)]),
                      *(parse(c.text) for c in _below(surface, stack))):
            if value is not None:
                return value
    return None
