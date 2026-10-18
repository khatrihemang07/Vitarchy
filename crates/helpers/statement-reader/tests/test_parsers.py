from datetime import date

import pytest

from statement_reader.metadata.parsers import count_value, mask_value, money_value, period_value, total_value
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()


def test_money_keeps_the_sign_of_a_balance_and_totals_are_always_positive():
    assert money_value("4.00CR", V) == 400 and money_value("5.00 DR", V) == -500
    assert total_value("- ₹1,696.06", V) == 169606 and total_value("(10.00)", V) == 1000
    assert total_value("n/a", V) is None


def test_counts_are_whole_numbers():
    assert count_value(" 12 ", V) == 12 and count_value("12.5", V) is None and count_value("", V) is None


@pytest.mark.parametrize("text,expected", [
    ("XXXXXX0123", "XXXXXX0123"), ("4000XXXXXXXX7777", "4000XXXXXXXX7777"), (": xxxxxxx1111", "xxxxxxx1111"),
    ("xxxx xxxx xxxx 9876", "xxxx xxxx xxxx 9876"), ("XXXX XXXX XXXX XX56 Pay", "XXXX XXXX XXXX XX56"),
    ("XXXXXX0555 ( INR )  - FAKE NAME", "XXXXXX0555"), ("20334174321", "20334174321"),
    ("Savings", None), ("12 Jun", None), ("", None),
])
def test_masks_are_read_as_printed(text, expected):
    assert mask_value(text, V) == expected


@pytest.mark.parametrize("text,expected", [
    ("June 29, 2026 to July 28, 2026", (date(2026, 6, 29), date(2026, 7, 28))),
    ("01-09-2026  To  30-09-2026", (date(2026, 9, 1), date(2026, 9, 30))),
    ("March 01, 2026 - April 30, 2026", (date(2026, 3, 1), date(2026, 4, 30))),
    ("18 Jul 26 to 17 Aug 26", (date(2026, 7, 18), date(2026, 8, 17))),
    ("19 Jul, 2026 - 18 Aug, 2026", (date(2026, 7, 19), date(2026, 8, 18))),
])
def test_periods_split_on_the_vocabulary_separators(text, expected):
    assert period_value(text, V) == expected


def test_both_ends_of_a_period_are_read_in_the_same_format():
    assert period_value("08/01/2026 to 08/31/2026", V) == (date(2026, 8, 1), date(2026, 8, 31))
    assert period_value("01/08/2026 to 31/08/2026", V) == (date(2026, 8, 1), date(2026, 8, 31))


def test_the_first_masked_number_anywhere_is_found():
    from statement_reader.metadata.parsers import first_mask

    assert first_mask("Bank - Account XXXXXX0911 and more", V) == "XXXXXX0911"
    assert first_mask("nothing masked", V) is None


def test_text_that_is_not_a_period_gives_none():
    assert period_value("for the period", V) is None and period_value("a to b", V) is None
