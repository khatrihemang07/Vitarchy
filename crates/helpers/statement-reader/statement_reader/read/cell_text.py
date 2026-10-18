"""Cell values as displayed text: pure helpers shared by the spreadsheet readers."""

from __future__ import annotations

import re
from datetime import date, datetime, time

DECIMALS = re.compile(r"0\.(0+)")


def datetime_text(value: datetime | date | time) -> str:
    """ISO text: a date when there is no time of day, a time alone when there is no date."""
    if isinstance(value, datetime):
        return value.date().isoformat() if value.time() == time() else value.isoformat()
    return value.isoformat()


def number_text(value: float | int, number_format: str = "General") -> str:
    """A number as Excel would show it: decimals and thousands commas follow the format."""
    match = DECIMALS.search(number_format)
    if match:
        places = len(match.group(1))
        commas = "#,##" in number_format or "0,0" in number_format
        return f"{value:,.{places}f}" if commas else f"{value:.{places}f}"
    if float(value).is_integer():
        return str(int(value))
    return format(value, ".15g")


def blank_merged_followers(grid: list[list[str]], ranges: list[tuple[int, int, int, int]]) -> None:
    """Keep a merged range's value only in its top-left cell. Ranges are 0-based, inclusive."""
    for row_lo, row_hi, col_lo, col_hi in ranges:
        for r in range(row_lo, min(row_hi, len(grid) - 1) + 1):
            for c in range(col_lo, min(col_hi, len(grid[r]) - 1) + 1):
                if (r, c) != (row_lo, col_lo):
                    grid[r][c] = ""


def pad_rows(rows: list[list[str]]) -> list[list[str]]:
    width = max((len(row) for row in rows), default=0)
    return [row + [""] * (width - len(row)) for row in rows]
