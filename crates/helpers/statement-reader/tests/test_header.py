from statement_reader.locate.header import find_header
from statement_reader.read.lines import group_lines
from statement_reader.read.pdf_words import Word
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()


def w(text, x0, y, width=None, bold=True):
    return Word(x0, y - 4, x0 + (width or 5 * len(text)), y + 4, text, bold)


def lines_of(*rows):
    """rows: lists of (text, x0) per line; y = 100 + 10 * index unless a (y, [...]) pair is given."""
    words = []
    for index, row in enumerate(rows):
        y, cells = row if isinstance(row, tuple) else (100 + 10 * index, row)
        for text, x0 in cells:
            x = x0
            for part in text.split(" "):
                words.append(w(part, x, y))
                x += 5 * len(part) + 3
    return group_lines(words)


def fields(header):
    return [c.field for c in header.layout.columns]


def test_a_single_line_with_a_date_and_money_columns_is_a_header():
    lines = lines_of([("Date", 40), ("Description", 110), ("Debit", 340), ("Credit", 410), ("Balance", 480)])
    header = find_header(lines, V)
    assert fields(header) == ["date", "description", "debit", "credit", "balance"]
    assert (header.first, header.last) == (0, 0)


def test_two_adjacent_lines_merge_into_two_line_labels():
    lines = lines_of(
        [("S No.", 40), ("Transaction", 90), ("Transaction Remarks", 160), ("Withdrawal", 400), ("Deposit", 470)],
        [("Date", 90), ("Amount (INR)", 400), ("Amount (INR)", 470), ("Balance", 540)],
    )
    header = find_header(lines, V)
    assert fields(header) == ["serial", "date", "description", "debit", "credit", "balance"]
    assert (header.first, header.last) == (0, 1)


def test_lines_with_no_date_or_no_money_are_not_headers():
    assert find_header(lines_of([("Name", 40), ("Description", 110), ("Total", 340)]), V) is None
    assert find_header(lines_of([("Date", 40), ("Description", 110), ("Remarks", 340)]), V) is None


def test_two_concepts_are_not_enough():
    assert find_header(lines_of([("Date", 40), ("Amount", 340)]), V) is None


def test_credit_debit_instrument_is_an_instrument_never_a_credit_or_debit_column():
    lines = lines_of([("Date", 40), ("Credit/debit instrument", 110), ("Amount", 340), ("Details", 250)])
    assert fields(find_header(lines, V)) == ["date", "instrument", "description", "amount"]


def test_a_body_row_below_a_header_is_not_merged_into_it():
    lines = lines_of(
        [("Date", 40), ("Details", 110), ("Debit", 340), ("Credit", 410), ("Balance", 480)],
        [("01/03/2026", 40), ("UPI/12", 110), ("500.00", 340), ("9,500.00", 480)],
    )
    header = find_header(lines, V)
    assert (header.first, header.last) == (0, 0)


def test_header_labels_spread_over_three_lines_still_find_date_and_amount():
    lines = lines_of(
        (100, [("Transaction Details", 180), ("Amount", 372)]),
        (103.5, [("Date", 35)]),
        (114, [("for Statement Period: 18 Jul 26", 158)]),
    )
    header = find_header(lines, V)
    assert fields(header) == ["date", "description", "amount"]
    assert (header.first, header.last) == (0, 1)


def test_unrelated_text_on_the_header_line_is_ignored():
    lines = lines_of([("SPENDS OVERVIEW", 89), ("Date", 208), ("SerNo.", 262), ("Transaction Details", 305),
                      ("Amount (in`)", 500)])
    assert fields(find_header(lines, V)) == ["date", "source_id", "description", "amount"]


def test_the_search_can_start_below_an_earlier_header():
    rows = [[("Date", 40), ("Details", 110), ("Debit", 340)], [("x", 40)],
            [("Date", 40), ("Details", 110), ("Credit", 340)]]
    lines = lines_of(*rows)
    assert find_header(lines, V, start=1).first == 2


def test_a_label_missing_from_the_vocabulary_leaves_no_description_so_there_is_no_header():
    assert find_header(lines_of([("Date", 40), ("Gibberish", 110), ("Debit", 340), ("Balance", 400)]), V) is None
    found = find_header(lines_of([("Date", 40), ("Particulars", 110), ("Debit", 340), ("Balance", 400)]), V)
    assert "description" in [c.field for c in found.layout.columns]


def test_a_table_without_row_text_is_not_a_transaction_table():
    assert find_header(lines_of([("Account", 40), ("Date", 110), ("Amount", 340)]), V) is None
