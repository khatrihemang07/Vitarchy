from datetime import date

from statement_reader.metadata.periods import date_after, fill_period
from statement_reader.metadata.surface import FactCell, Surface
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()


def surface(*texts):
    return Surface([FactCell(t, 0, 1, float(i * 20)) for i, t in enumerate(texts)], 40.0, 2.5)


def test_the_date_straight_after_a_label_is_read_in_any_vocabulary_format():
    s = surface("Your Opening Balance on 01-08-26: 88565.74", "As on 31-08-26")
    assert date_after(s, ("your opening balance on",), V) == date(2026, 8, 1)
    assert date_after(s, ("as on",), V) == date(2026, 8, 31)
    assert date_after(s, ("nothing",), V) is None


def test_only_the_missing_ends_of_a_period_are_filled():
    s = surface("Your Opening Balance on 01-08-26:", "As on 31-08-26")
    assert fill_period((None, None), s, V) == (date(2026, 8, 1), date(2026, 8, 31))
    assert fill_period((date(2026, 7, 1), None), s, V) == (date(2026, 7, 1), date(2026, 8, 31))
