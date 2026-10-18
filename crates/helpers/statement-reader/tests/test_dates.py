from datetime import date

import pytest

from statement_reader.normalize.dates import (
    date_prefix_words,
    detect_format,
    parse_any,
    parse_date,
    parse_time,
)
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()


@pytest.mark.parametrize("text,expected", [
    ("31/08/2026", date(2026, 8, 31)), ("01.03.2026", date(2026, 3, 1)), ("01-08-26", date(2026, 8, 1)),
    ("02 Jul 26", date(2026, 7, 2)), ("19 Jul, 2026", date(2026, 7, 19)), ("Oct 18, 2026", date(2026, 10, 18)),
    ("18 Sept, 2026", date(2026, 9, 18)), ("June 29, 2026", date(2026, 6, 29)),
    ("2026-09-01", date(2026, 9, 1)), ("2026-09-01T10:30:00", date(2026, 9, 1)),
    ("  01/03/2026  ", date(2026, 3, 1)), ("5 March 2026", date(2026, 3, 5)),
])
def test_every_printed_date_shape_in_the_reports_parses(text, expected):
    found = parse_any(text, V)
    assert found is not None and found[0] == expected


@pytest.mark.parametrize("text", ["", "Total", "1,018.00", "UPI/1234", "99/99/2026", "12345678901234"])
def test_text_that_is_not_a_date_does_not_parse(text):
    assert parse_any(text, V) is None


def test_a_column_of_day_first_values_settles_on_day_first():
    assert detect_format(["01/02/2026", "03/04/2026"], V) == "%d/%m/%Y"


def test_one_day_above_twelve_rules_out_month_first():
    assert detect_format(["01/02/2026", "13/02/2026"], V) == "%d/%m/%Y"
    assert detect_format(["02/13/2026", "01/14/2026"], V) == "%m/%d/%Y"


def test_the_statement_period_settles_an_ambiguous_column():
    values = ["03/09/2026", "03/10/2026"]
    assert detect_format(values, V) == "%d/%m/%Y"
    assert detect_format(values, V, (date(2026, 3, 1), date(2026, 3, 31))) == "%m/%d/%Y"


def test_two_digit_years_and_month_names_and_sept_are_detected_per_column():
    assert detect_format(["01-08-26", "30-08-26"], V) == "%d-%m-%y"
    assert detect_format(["02 Jul 26", "31 Jul 26"], V) == "%d %b %y"
    assert detect_format(["18 Sept, 2026", "19 Oct, 2026"], V) == "%d %b, %Y"


def test_a_column_that_mixes_shapes_has_no_format():
    assert detect_format(["01/02/2026", "02 Jul 26"], V) is None
    assert detect_format([], V) is None


def test_parse_date_uses_the_given_format_only():
    assert parse_date("01/02/2026", "%d/%m/%Y", V) == date(2026, 2, 1)
    assert parse_date("01/02/2026", "%m/%d/%Y", V) == date(2026, 1, 2)
    assert parse_date("xx", "%d/%m/%Y", V) is None


@pytest.mark.parametrize("words,count", [
    (["31/08/2026", "51234567958", "Examplepay"], 1), (["01", "Aug", "26", "PAYMENT", "RECEIVED"], 3),
    (["Oct", "18,", "2026", "x"], 3), (["PAYMENT", "01"], 0), ([], 0),
])
def test_the_words_that_make_up_a_leading_date_are_counted(words, count):
    assert date_prefix_words(words, V) == count


def test_times_parse_from_twelve_and_twenty_four_hour_text():
    assert parse_time("11:20 am", V).isoformat() == "11:20:00"
    assert parse_time("11:20 PM", V).isoformat() == "23:20:00"
    assert parse_time("23:59", V).isoformat() == "23:59:00"
    assert parse_time("noon", V) is None
