import pytest

from statement_reader.normalize.amounts import balance_paise, column_amount, parse_money
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()


@pytest.mark.parametrize("text,paise", [
    ("1,23,456.78", 12345678), ("₹1,269", 126900), ("`2,903.92", 290392), ("Rs. 5,000", 500000),
    ("INR 12.5", 1250), ("500", 50000), ("0.00", 0), ("12.345", 1235), ("1,000.00", 100000),
    ("3,00,000.00", 30000000), ("  ₹ 7,485.72 ", 748572),
])
def test_amounts_are_exact_integer_paise_whatever_the_grouping_or_currency_mark(text, paise):
    assert parse_money(text, V).paise == paise


@pytest.mark.parametrize("text,direction", [
    ("18,796.39 CR", "credit"), ("10,257.00 C", "credit"), ("4.00CR", "credit"), ("453.40 D", "debit"),
    ("100.00 Dr", "debit"), ("100.00", None),
])
def test_cr_and_dr_suffixes_attached_or_separate_give_a_direction_hint(text, direction):
    assert parse_money(text, V).direction == direction


@pytest.mark.parametrize("text", ["(1,000.00)", "-500", "−500.25", "-₹1,200"])
def test_parentheses_and_a_minus_sign_make_an_amount_negative(text):
    assert parse_money(text, V).negative


def test_a_sign_separated_by_a_space_is_an_operator_not_a_negative():
    assert not parse_money("- ₹1,696.06", V).negative
    assert parse_money("+ ₹1,269", V).paise == 126900


@pytest.mark.parametrize("text", ["", "-", "--", "abc", "12/08/2026", "1,2,", "UPI", "5 Jul"])
def test_text_that_is_not_an_amount_is_none(text):
    assert parse_money(text, V) is None


def test_a_debit_or_credit_column_cell_without_money_in_it_is_none():
    assert column_amount("-", V) is None and column_amount("0.00", V) is None
    assert column_amount("", V) is None and column_amount("0", V) is None
    assert column_amount("1,018.00 CR", V) == 101800


def test_balances_are_signed_negative_for_dr_or_a_minus_sign():
    assert balance_paise("38,521.89CR", V) == 3852189
    assert balance_paise("2,000.00 DR", V) == -200000
    assert balance_paise("-20.40", V) == -2040
    assert balance_paise("", V) is None and balance_paise("-", V) is None
