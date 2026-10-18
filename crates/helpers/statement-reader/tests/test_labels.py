from statement_reader.metadata.labels import find_value
from statement_reader.metadata.parsers import money_value, period_value
from statement_reader.metadata.surface import FactCell, Surface
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()


def surface(*cells, below=40.0):
    return Surface([FactCell(*c) for c in cells], below, 2.5)


def money(text):
    return money_value(text, V)


def test_a_value_after_a_colon_in_the_same_cell_is_found():
    s = surface(("Your Opening Balance on 01-08-26: 88565.74", 40, 240, 100.0))
    assert find_value(s, ("your opening balance",), money) == 8856574


def test_a_value_to_the_right_on_the_same_line_is_found():
    s = surface(("Your Opening Balance on 01-08-26:", 40, 200, 100.0), ("5,000.00", 240, 270, 101.0))
    assert find_value(s, ("your opening balance",), money) == 500000


def test_a_value_on_the_line_below_under_the_label_is_found_and_other_columns_are_ignored():
    s = surface(("Previous Balance", 30, 82, 100.0), ("Purchases", 130, 170, 100.0),
                ("1,000.00", 36, 72, 116.0), ("2,000.00", 140, 170, 116.0))
    assert find_value(s, ("previous balance",), money) == 100000
    assert find_value(s, ("purchases",), money) == 200000


def test_a_label_wrapped_over_stacked_lines_is_joined_before_matching():
    s = surface(("Payments,", 123, 156, 100.0), ("Reversals & other", 112, 141, 108.0),
                ("Credits ( ` )", 129, 167, 116.0), ("10,257.00", 119, 155, 140.0))
    assert find_value(s, ("payments, reversals & other credits",), money) == 1025700


def test_the_longest_label_and_not_a_longer_word_decide_a_match():
    s = surface(("Purchasesx", 30, 80, 100.0), ("9.00", 30, 50, 110.0))
    assert find_value(s, ("purchases",), money) is None


def test_a_value_too_far_below_is_not_taken():
    s = surface(("Total Amount Due", 30, 100, 100.0), ("5.00", 30, 50, 300.0))
    assert find_value(s, ("total amount due",), money) is None


def test_the_first_label_that_has_a_value_wins_in_reading_order():
    s = surface(("Closing Balance", 30, 90, 100.0), ("Closing Balance", 30, 90, 300.0), ("7.00", 30, 40, 310.0))
    assert find_value(s, ("closing balance",), money) == 700


def test_period_labels_may_sit_in_the_middle_of_a_sentence_and_split_across_cells():
    s = surface(("Statement of Transactions in Savings Account X1 for the period March 01, 2026 - April 30, 2026",
                 37, 500, 90.0))
    assert find_value(s, ("for the period",), lambda t: period_value(t, V), contains=True)[1].month == 4
    sheet = Surface([FactCell("Transaction Date from", 1, 1, 4), FactCell("01/07/2026", 3, 3, 4),
                     FactCell("to", 4, 4, 4), FactCell("31/07/2026", 5, 5, 4)], 2.0, 0.1, True)
    assert find_value(sheet, ("transaction date from",), lambda t: period_value(t, V), contains=True)[0].day == 1


def test_sheet_cells_find_the_value_under_the_label_in_the_same_column():
    sheet = Surface([FactCell("Dr Count", 1, 1, 10), FactCell("Cr Count", 2, 2, 10), FactCell("6", 1, 1, 11),
                     FactCell("4", 2, 2, 11)], 2.0, 0.1, True)
    assert find_value(sheet, ("cr count",), lambda t: int(t) if t.isdigit() else None) == 4


def test_a_sheet_surface_can_leave_out_the_tables_own_header_row():
    from statement_reader.metadata.surface import sheet_surface

    grid = [["Closing Balance", "Date"], ["10.00", "01/01/2026"], ["Closing Balance", ""], ["99.00", ""]]
    full = sheet_surface(grid)
    cut = sheet_surface(grid, range(0, 1))
    assert len(full.cells) == len(cut.cells) + 2
    assert find_value(cut, ("closing balance",), money) == 9900
