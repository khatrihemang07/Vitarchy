from datetime import date

import pytest

from statement_reader.normalize.amounts import parse_foreign
from statement_reader.normalize.rows import NormaliseError, build_transactions
from statement_reader.segment.rows import RawRow
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()


def row(description="x", card=None, pseudo=None, **cells):
    return RawRow(cells, description, 0, pseudo, card)


def test_separate_debit_and_credit_columns_with_a_running_balance():
    rows = [row("A", date="01/03/2026", debit="500.00", balance="9,500.00"),
            row("UPI/600000000000/B", date="02/03/2026", credit="200.00", debit="-", balance="9,700.00")]
    built = build_transactions(rows, V, kind="savings")
    first, second = built.transactions
    assert (first.date, first.amount_paise, first.direction, first.balance_paise) == ("2026-03-01", 50000, "debit", 950000)
    assert (second.direction, second.reference) == ("credit", "600000000000")


def test_a_single_amount_column_uses_suffix_type_word_and_defaults_to_debit_for_cards():
    rows = [row("PAY", date="01/08/2026", amount="10,257.00 C"), row("SHOP", date="02/08/2026", amount="453.40 D"),
            row("PLAIN", date="03/08/2026", amount="5.00"), row("TYPED", date="04/08/2026", amount="7", type="CREDIT")]
    got = [(t.direction, t.amount_paise) for t in build_transactions(rows, V, kind="credit_card").transactions]
    assert got == [("credit", 1025700), ("debit", 45340), ("debit", 500), ("credit", 700)]


def test_an_opening_row_gives_the_opening_balance_and_is_not_a_transaction():
    rows = [row("B/F", pseudo="opening", date="01/03/2026", balance="30,000.00"),
            row("A", date="02/03/2026", debit="5.00", balance="29,995.00")]
    built = build_transactions(rows, V, kind="savings")
    assert built.opening_paise == 3000000 and len(built.transactions) == 1


def test_the_balance_change_decides_a_single_amount_with_no_other_hint_in_either_polarity():
    rows = [row("A", date="01/03/2026", amount="300.00", balance="1,300.00"),
            row("B", date="02/03/2026", amount="100.00", balance="1,200.00")]
    got = [t.direction for t in build_transactions(rows, V, kind="savings", opening_paise=100000).transactions]
    assert got == ["credit", "debit"]
    loan = [row("A", date="01/03/2026", amount="300.00", balance="700.00"),
            row("B", date="02/03/2026", amount="100.00", balance="800.00")]
    got = [t.direction for t in build_transactions(loan, V, kind="loan", opening_paise=100000).transactions]
    assert got == ["credit", "debit"]


def test_a_signed_amount_follows_the_kind_of_account():
    rows = [row("A", date="01/03/2026", amount="-500.00"), row("B", date="02/03/2026", amount="200.00")]
    assert [t.direction for t in build_transactions(rows, V, kind="savings").transactions] == ["debit", "debit"]
    assert [t.direction for t in build_transactions(rows, V, kind="credit_card").transactions] == ["credit", "debit"]


def test_dates_settle_on_one_format_for_the_whole_column_using_the_period():
    rows = [row("A", date="03/09/2026", debit="1.00"), row("B", date="03/10/2026", debit="1.00")]
    period = (date(2026, 3, 1), date(2026, 3, 31))
    built = build_transactions(rows, V, kind="savings", period=period)
    assert [t.date for t in built.transactions] == ["2026-03-09", "2026-03-10"]


def test_unreadable_dates_are_an_error_with_a_code():
    with pytest.raises(NormaliseError) as error:
        build_transactions([row("A", date="whenever", debit="1.00")], V, kind="savings")
    assert error.value.code == "unreadable_dates"


def test_rows_without_an_amount_are_left_out_and_counted():
    rows = [row("A", date="01/03/2026", debit="0.00", credit="-"), row("B", date="01/03/2026", debit="4.00")]
    built = build_transactions(rows, V, kind="savings")
    assert len(built.transactions) == 1 and built.skipped == 1


def test_source_id_foreign_amount_and_utr_columns_fill_their_fields():
    rows = [row("S", date="01/03/2026", amount="2,075.00", source_id="14000000007", utr="123456789012",
                foreign_amount="USD 25.00")]
    (t,) = build_transactions(rows, V, kind="credit_card").transactions
    assert (t.source_id, t.reference, t.foreign_amount, t.foreign_currency) == ("14000000007", "123456789012", "25.00", "USD")


def test_a_dash_in_a_source_id_cell_is_no_id():
    (t,) = build_transactions([row("S", date="01/03/2026", debit="1", source_id="-")], V, kind="savings").transactions
    assert t.source_id is None


@pytest.mark.parametrize("text,expected", [("USD 25.00", ("25.00", "USD")), ("EUR 1,234.50", ("1234.50", "EUR")),
                                           ("25.00", ("25.00", None)), ("-", None), ("", None)])
def test_foreign_amounts_split_into_amount_and_currency(text, expected):
    assert parse_foreign(text) == expected
