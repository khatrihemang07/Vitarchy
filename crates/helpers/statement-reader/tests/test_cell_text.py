from datetime import date, datetime, time

import pytest

from statement_reader.read.cell_text import (
    blank_merged_followers,
    datetime_text,
    number_text,
    pad_rows,
)


def test_a_date_without_time_of_day_is_iso_date():
    assert datetime_text(datetime(2026, 9, 1)) == "2026-09-01"
    assert datetime_text(date(2026, 9, 1)) == "2026-09-01"


def test_a_date_with_time_of_day_is_iso_datetime_and_a_bare_time_is_a_time():
    assert datetime_text(datetime(2026, 9, 1, 11, 20)) == "2026-09-01T11:20:00"
    assert datetime_text(time(11, 20)) == "11:20:00"


@pytest.mark.parametrize("value,fmt,expected", [
    (5500.0, "General", "5500"), (12.5, "General", "12.5"), (3, "General", "3"),
    (5500.5, "0.00", "5500.50"), (1234567.891, "#,##0.00", "1,234,567.89"),
    (0.1 + 0.2, "General", "0.3"),
])
def test_numbers_follow_their_format(value, fmt, expected):
    assert number_text(value, fmt) == expected


def test_a_merged_range_keeps_only_its_top_left_value():
    grid = [["Amount", "Amount", "x"], ["Amount", "Amount", "y"]]
    blank_merged_followers(grid, [(0, 1, 0, 1)])
    assert grid == [["Amount", "", "x"], ["", "", "y"]]


def test_merged_ranges_past_the_grid_are_clipped():
    grid = [["a"]]
    blank_merged_followers(grid, [(0, 5, 0, 5)])
    assert grid == [["a"]]


def test_rows_are_padded_to_the_widest():
    assert pad_rows([["a"], ["b", "c"]]) == [["a", ""], ["b", "c"]]
    assert pad_rows([]) == []
