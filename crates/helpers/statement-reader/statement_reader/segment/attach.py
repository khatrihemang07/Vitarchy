"""Which row a non-date line belongs to: the nearest anchor by vertical distance.

Cells are often vertically centred, so a row's first narration line sits ABOVE its date line.
"""

from __future__ import annotations

from ..read.lines import Line
from .balance import balance_gaps

TIE = 0.5  # points: distances closer than this are a tie


def _owner(lines: list[Line], index: int, candidates: list[int]) -> int:
    y = lines[index].y
    ranked = sorted(candidates, key=lambda a: abs(lines[a].y - y))
    best = ranked[0]
    if len(ranked) > 1 and abs(abs(lines[ranked[1]].y - y) - abs(lines[best].y - y)) < TIE:
        # Equidistant: a bold line after a regular one starts a new cell, so it goes to the row below.
        starts_cell = lines[index].bold and (index == 0 or not lines[index - 1].bold)
        pair = sorted([best, ranked[1]])
        return pair[1] if starts_cell else pair[0]
    return best


def _contiguous(lines: list[Line], anchor: int, step: int, limit: float) -> set[int]:
    """Lines walking away from `anchor` while each is within `limit` of the one before."""
    kept: set[int] = set()
    previous, index = lines[anchor].y, anchor + step
    while 0 <= index < len(lines) and abs(lines[index].y - previous) <= limit:
        kept.add(index)
        previous, index = lines[index].y, index + step
    return kept


def attach_lines(lines: list[Line], anchors: set[int], pseudo: set[int], limit: float,
                 texty: set[int] | None = None) -> dict[int, int]:
    """line index -> owning anchor index. Anchors and lines that are part of no row are omitted.

    `texty` are the anchors whose own line carries description text.
    Pseudo anchors (B/F, opening balance) never own narration. Above the first and below the last
    anchor only a chain of lines no further apart than `limit` is kept.
    """
    if not anchors:
        return {}
    ordered = sorted(anchors)
    owners = [a for a in ordered if a not in pseudo] or ordered
    keep = _contiguous(lines, ordered[0], -1, limit) | _contiguous(lines, ordered[-1], +1, limit)
    result: dict[int, int] = {}
    for index in range(len(lines)):
        outside = index < ordered[0] or index > ordered[-1]
        if index not in anchors and (not outside or index in keep):
            result[index] = _owner(lines, index, owners)
    balance_gaps(lines, owners, texty or set(), result)
    _split_at_bold(lines, owners, result)
    return result


def _bold_start(lines: list[Line], index: int, anchors: set[int]) -> bool:
    return lines[index].bold and (index == 0 or index - 1 in anchors or not lines[index - 1].bold)


def _split_at_bold(lines: list[Line], owners: list[int], result: dict[int, int]) -> None:
    """Where most rows open with a bold line, that line starts its row's text: everything between
    two anchors from the last bold start on belongs to the lower anchor, the rest to the upper one.
    (Nearest-anchor fails for top-aligned multi-line rows whose anchor sits beside their first lines.)"""
    gaps = [(upper, lower, [i for i in range(upper + 1, lower) if i in result])
            for upper, lower in zip(owners, owners[1:], strict=False)]
    starts = [[i for i in between if _bold_start(lines, i, set(owners))] for _, _, between in gaps]
    if not gaps or sum(1 for s in starts if s) * 2 < len(gaps):
        return
    for (upper, lower, between), found in zip(gaps, starts, strict=True):
        if found:
            for i in between:
                result[i] = lower if i >= found[-1] else upper
