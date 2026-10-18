from statement_reader.locate.layout import Column, Layout
from statement_reader.read.lines import group_lines
from statement_reader.read.pdf_words import Word
from statement_reader.segment.rows import segment_region
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()
LAYOUT = Layout([Column("date", 40, 80), Column("description", 144, 300), Column("credit", 340, 407),
                 Column("debit", 420, 486), Column("balance", 500, 559)])


def build(*rows, page=0):
    """rows: (y, [(text, x0, bold)]) with words spaced by 3 points."""
    words = []
    for y, cells in rows:
        for cell in cells:
            text, x0, *bold = cell
            x = x0
            for part in text.split(" "):
                words.append(Word(x, y - 4, x + 5 * len(part), y + 4, part, bool(bold and bold[0])))
                x += 5 * len(part) + 3
    return group_lines(words, page)


def test_an_anchor_with_amounts_becomes_a_row_with_cells_and_description():
    lines = build((100, [("01/03/2026", 40), ("UPI/1 PAY", 144), ("500.00", 440), ("9,500.00", 510)]))
    (row,) = segment_region(lines, LAYOUT, V).rows
    assert row.cells == {"date": "01/03/2026", "debit": "500.00", "balance": "9,500.00"}
    assert row.description == "UPI/1 PAY" and row.pseudo is None


def test_narration_above_and_below_the_date_line_is_attached_in_order():
    lines = build(
        (100, [("PAYEE NAME", 144, True)]),
        (109, [("01/03/2026", 40), ("UPI/600000000000/NA/fake0@", 144), ("500.00", 440), ("9,500.00", 510)]),
        (118, [("exb/Example", 144)]),
        (136, [("NEXT", 144, True)]),
        (145, [("02/03/2026", 40), ("UPI/2", 144), ("5.00", 440), ("9,495.00", 510)]),
    )
    rows = segment_region(lines, LAYOUT, V).rows
    assert [r.description for r in rows] == ["PAYEE NAME UPI/600000000000/NA/fake0@exb/Example", "NEXT UPI/2"]


def test_an_opening_balance_row_is_pseudo_and_attracts_no_narration():
    lines = build(
        (100, [("01/03/2026", 40), ("B/F", 144), ("30,000.00", 510)]),
        (109, [("PAYEE", 144, True)]),
        (118, [("02/03/2026", 40), ("UPI/2", 144), ("5.00", 440), ("29,995.00", 510)]),
    )
    first, second = segment_region(lines, LAYOUT, V).rows
    assert first.pseudo == "opening" and first.description == "B/F"
    assert second.description == "PAYEE UPI/2"


def test_a_masked_card_alone_on_a_line_sets_the_card_for_the_rows_after_it_and_carries_on():
    lines = build(
        (100, [("4000XXXXXXXX1111", 40)]),
        (110, [("01/03/2026", 40), ("A", 144), ("5.00", 440)]),
        (125, [("4000XXXXXXXX2222", 40)]),
        (135, [("02/03/2026", 40), ("B", 144), ("6.00", 440)]),
    )
    result = segment_region(lines, LAYOUT, V)
    assert [r.card for r in result.rows] == ["4000XXXXXXXX1111", "4000XXXXXXXX2222"]
    assert result.card == "4000XXXXXXXX2222"
    again = segment_region(build((100, [("03/03/2026", 40), ("C", 144), ("7.00", 440)])), LAYOUT, V,
                           card=result.card)
    assert again.rows[0].card == "4000XXXXXXXX2222"


def test_a_cardholder_divider_is_not_description_and_splits_the_rows_around_it():
    lines = build(
        (100, [("01/03/2026", 40), ("PAYMENT", 144), ("5.00", 440)]),
        (110, [("TRANSACTIONS FOR SOME NAME", 144)]),
        (120, [("02/03/2026", 40), ("SHOP", 144), ("6.00", 440)]),
    )
    rows = segment_region(lines, LAYOUT, V).rows
    assert [r.description for r in rows] == ["PAYMENT", "SHOP"]


def test_a_date_line_without_any_amount_is_not_a_row():
    lines = build((100, [("01/03/2026", 40), ("As on date", 144)]))
    assert segment_region(lines, LAYOUT, V).rows == []


def test_noise_lines_and_opening_balance_text_lines_are_skipped():
    lines = build(
        (100, [("Your Opening Balance on 01-08-26:", 40), ("100.00", 510)]),
        (110, [("for Statement Period: 18 Jul 26 to 17 Aug 26", 144)]),
        (120, [("01/03/2026", 40), ("ROW", 144), ("5.00", 440)]),
    )
    (row,) = segment_region(lines, LAYOUT, V).rows
    assert row.description == "ROW"


def test_a_bold_first_line_is_never_glued_to_the_line_after_it():
    lines = build(
        (100, [("XX.rzp@rxdemo", 144, True)]),
        (109, [("01/03/2026", 40), ("UPI/XX.rzp@rxdemo/Pay Via", 144), ("500.00", 440), ("9,500.00", 510)]),
    )
    (row,) = segment_region(lines, LAYOUT, V).rows
    assert row.description == "XX.rzp@rxdemo UPI/XX.rzp@rxdemo/Pay Via"


def test_a_figures_only_line_without_its_label_is_not_narration():
    lines = build(
        (100, [("88565.74", 184)]),
        (115, [("01/03/2026", 40), ("UPI/1", 144), ("500.00", 440), ("9,500.00", 510)]),
    )
    (row,) = segment_region(lines, LAYOUT, V).rows
    assert row.description == "UPI/1"


def test_text_in_a_type_column_stays_in_the_description_unless_it_is_only_a_direction_word():
    layout = Layout([Column("date", 40, 80), Column("type", 90, 130), Column("description", 144, 300),
                     Column("debit", 420, 486), Column("balance", 500, 559)])
    lines = build(
        (100, [("01/03/2026", 40), ("NET BANKING", 90), ("UPI/1", 144), ("5.00", 440), ("95.00", 510)]),
        (120, [("02/03/2026", 40), ("DEBIT", 90), ("UPI/2", 144), ("5.00", 440), ("90.00", 510)]),
    )
    rows = segment_region(lines, layout, V).rows
    assert [r.description for r in rows] == ["NET BANKING UPI/1", "UPI/2"]


def test_the_wrap_width_ignores_a_few_stray_wide_lines():
    from statement_reader.segment.rows import _wrap_width

    assert _wrap_width([300.0] * 20 + [496.0, 0.0]) == 300.0
    assert _wrap_width([]) == 0.0
