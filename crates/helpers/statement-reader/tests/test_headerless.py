from statement_reader.locate.headerless import infer_layouts
from statement_reader.read.lines import group_lines
from statement_reader.read.pdf_words import Word
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()


def words(rows):
    out = []
    for y, cells in rows:
        for text, x1 in cells:
            out.append(Word(x1 - 5 * len(text), y - 4, x1, y + 4, text))
    return group_lines(out)


def ledger(with_ref=False):
    rows = []
    for i in range(3):
        y = 100 + 20 * i
        cells = [("01/03/2026", 80), ("NARRATION TEXT", 250)]
        if with_ref:
            cells.append(("-", 330))
        cells += [("-", 395), ("12.00", 465), (f"{100 + i}.00", 550)] if i % 2 else \
                 [("5.00", 395), ("-", 465), (f"{100 + i}.00", 550)]
        rows.append((y, cells))
    return words(rows)


def test_two_unknown_money_columns_give_both_orders_with_the_last_as_the_balance():
    first, second = infer_layouts(ledger(), V)
    assert [c.field for c in first.columns] == ["date", "description", "debit", "credit", "balance"]
    assert [c.field for c in second.columns] == ["date", "description", "credit", "debit", "balance"]


def test_a_column_that_only_holds_dashes_is_ignored_not_money():
    (first, _) = infer_layouts(ledger(with_ref=True), V)
    assert "ignored" in first.fields and len(first.fields & {"debit", "credit"}) == 2


def test_a_single_amount_column_plus_balance_gives_one_layout():
    lines = words([(100 + 20 * i, [("01/03/2026", 80), ("X", 250), (f"{5 + i}.00", 465), (f"{100 + i}.00", 550)])
                   for i in range(3)])
    (layout,) = infer_layouts(lines, V)
    assert [c.field for c in layout.columns] == ["date", "description", "amount", "balance"]


def test_text_that_is_not_a_ledger_gives_no_layout():
    assert infer_layouts(words([(100, [("hello", 50)]), (120, [("01/03/2026", 80), ("5.00", 200)])]), V) == []
