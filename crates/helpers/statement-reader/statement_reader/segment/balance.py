"""Splits the lines between two row anchors so that each row's text is centred on its anchor.

A vertically centred cell puts its narration lines symmetrically around the row's date and amount
line. Nearest-anchor gets that wrong when a row has several lines (its top lines sit nearer the row
above). For every gap we choose how many lines go to the upper row so that the summed distance
between each row's text centre and its anchor is smallest; plain distance only breaks ties.
"""

from __future__ import annotations

from collections.abc import Callable

from ..read.lines import Line

DISTANCE_WEIGHT = 0.001
EMPTY_ROW = 50.0  # cost of a figures-only row left without any text line
RowCost = Callable[[int, int, int], float]  # (anchor number, lines above, lines below) -> cost


def best_splits(sizes: list[int], row_cost: RowCost, tie_break: list[list[float]]) -> list[int]:
    """For each gap, how many of its lines go to the upper anchor.

    Anchor j sits between gap j-1 (it takes the gap's last `size - s` lines) and gap j (it takes the
    first `s`); anchors 0 and k have a fixed outside on one side, handled by `row_cost`.
    """
    if not sizes:
        return []
    cost: list[list[float]] = []
    back: list[list[int]] = []
    for j, size in enumerate(sizes):
        cost.append([])
        back.append([])
        for s in range(size + 1):
            if j == 0:
                options = [(row_cost(0, -1, s), 0)]
            else:
                options = [(cost[j - 1][p] + row_cost(j, sizes[j - 1] - p, s), p)
                           for p in range(sizes[j - 1] + 1)]
            best, arg = min(options)
            cost[j].append(best + tie_break[j][s])
            back[j].append(arg)
    final = [cost[-1][s] + row_cost(len(sizes), sizes[-1] - s, -1) for s in range(sizes[-1] + 1)]
    s = min(range(len(final)), key=lambda k: (final[k], -k))  # on a tie the upper row keeps the line
    chosen = [s]
    for j in range(len(sizes) - 1, 0, -1):
        s = back[j][s]
        chosen.append(s)
    return chosen[::-1]


def balance_gaps(lines: list[Line], owners: list[int], texty: set[int], result: dict[int, int]) -> None:
    """Re-split every gap between consecutive owners for the best-centred rows."""
    if len(owners) < 2:
        return
    pairs = list(zip(owners, owners[1:], strict=False))
    gaps = [[i for i in range(u + 1, lo) if i in result] for u, lo in pairs]
    outside_above = sorted(i for i in result if i < owners[0])
    outside_below = sorted(i for i in result if i > owners[-1])

    def row_cost(j: int, above: int, below: int) -> float:
        up = outside_above if above < 0 else gaps[j - 1][len(gaps[j - 1]) - above:]
        down = outside_below if below < 0 else gaps[j][:below]
        if owners[j] in texty and not up:
            return 0.0  # text on the anchor line with a tail below: a top-aligned row
        members = [lines[i].y for i in (*up, *down)] + ([lines[owners[j]].y] if owners[j] in texty else [])
        if not members:
            return EMPTY_ROW  # a row with no text at all is almost surely a misassignment
        return abs(sum(members) / len(members) - lines[owners[j]].y)

    tie = [[DISTANCE_WEIGHT * (sum(abs(lines[i].y - lines[u].y) for i in g[:s])
                               + sum(abs(lines[i].y - lines[lo].y) for i in g[s:]))
            for s in range(len(g) + 1)] for (u, lo), g in zip(pairs, gaps, strict=True)]
    splits = best_splits([len(g) for g in gaps], row_cost, tie)
    for (upper, lower), gap, s in zip(pairs, gaps, splits, strict=True):
        for position, index in enumerate(gap):
            result[index] = upper if position < s else lower
