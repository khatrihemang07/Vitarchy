import pytest

from statement_reader.normalize.amounts import Money
from statement_reader.normalize.direction import Figures, decide
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()


def test_separate_columns_decide_before_anything_else():
    assert decide(Figures(debit=500, type_text="CREDIT"), V) == ("debit", 500)
    assert decide(Figures(credit=700), V) == ("credit", 700)


def test_a_type_column_word_decides_a_single_amount():
    amount = Money(1018)
    assert decide(Figures(amount=amount, type_text="DEBIT"), V) == ("debit", 1018)
    assert decide(Figures(amount=amount, type_text="Cr"), V) == ("credit", 1018)
    assert decide(Figures(amount=amount, type_text="NET BANKING"), V) == ("debit", 1018)


def test_a_cr_or_c_suffix_makes_a_credit_and_a_plain_amount_is_a_debit():
    assert decide(Figures(amount=Money(100, direction="credit")), V) == ("credit", 100)
    assert decide(Figures(amount=Money(100, direction="debit")), V) == ("debit", 100)
    assert decide(Figures(amount=Money(100)), V) == ("debit", 100)


@pytest.mark.parametrize("negative_means,expected", [("debit", "debit"), ("credit", "credit")])
def test_a_negative_amount_follows_the_tables_sign_convention(negative_means, expected):
    figures = Figures(amount=Money(250, negative=True))
    assert decide(figures, V, negative_means=negative_means) == (expected, 250)


def test_with_no_other_hint_the_balance_change_decides():
    up = Figures(amount=Money(300), balance=1300, previous_balance=1000)
    down = Figures(amount=Money(300), balance=700, previous_balance=1000)
    assert decide(up, V, use_balance=True) == ("credit", 300)
    assert decide(down, V, use_balance=True) == ("debit", 300)
    assert decide(up, V, use_balance=True, inverted=True) == ("debit", 300)


def test_a_suffix_beats_the_balance_change():
    figures = Figures(amount=Money(300, direction="debit"), balance=1300, previous_balance=1000)
    assert decide(figures, V, use_balance=True) == ("debit", 300)


def test_a_row_with_no_amount_has_no_direction():
    assert decide(Figures(), V) is None
    assert decide(Figures(debit=0, credit=None), V) is None
