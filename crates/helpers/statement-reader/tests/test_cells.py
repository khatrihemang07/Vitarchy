from statement_reader.locate.layout import Column, Layout
from statement_reader.read.lines import Chunk
from statement_reader.segment.cells import assign, split_leading_date
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()
LAYOUT = Layout([
    Column("date", 40, 70), Column("type", 77, 101), Column("description", 144, 220),
    Column("credit", 370, 407), Column("debit", 440, 486), Column("balance", 530, 559)])


def texts(assigned):
    return {f: [c.text for c in chunks] for f, chunks in assigned.items()}


def test_chunks_land_in_the_column_they_overlap_and_text_defaults_to_the_description():
    chunks = [Chunk("12-09-2026", 37, 80), Chunk("NET BANKING", 74, 119), Chunk("VIN/DEMOBUS/xyz long text", 144, 330),
              Chunk("1,521.00", 450, 486), Chunk("68,053.62", 513, 559)]
    got = texts(assign(chunks, LAYOUT, V))
    assert got == {"date": ["12-09-2026"], "type": ["NET BANKING"],
                   "description": ["VIN/DEMOBUS/xyz long text"], "debit": ["1,521.00"], "balance": ["68,053.62"]}


def test_a_chunk_that_overlaps_no_column_is_description():
    assert texts(assign([Chunk("Examplefood", 192, 235)], LAYOUT, V)) == {"description": ["Examplefood"]}


def test_a_money_column_takes_only_money_or_a_blank_mark():
    got = texts(assign([Chunk("-", 450, 455), Chunk("hello", 445, 480)], LAYOUT, V))
    assert got == {"debit": ["-"], "description": ["hello"]}


def test_a_lone_cr_or_c_token_joins_the_amount_before_it():
    chunks = [Chunk("1,018.00", 524, 547), Chunk("CR", 552, 560)]
    layout = Layout([Column("description", 305, 400), Column("amount", 522, 557)])
    assert texts(assign(chunks, layout, V)) == {"amount": ["1,018.00 CR"]}


def test_a_money_chunk_just_outside_its_column_goes_to_the_column_that_starts_before_it():
    assert texts(assign([Chunk("5.00", 415, 430)], LAYOUT, V)) == {"credit": ["5.00"]}


def test_a_text_cell_starting_inside_a_narrow_column_belongs_to_it_even_when_wider():
    got = texts(assign([Chunk("NET BANKING", 92, 147), Chunk("PARTICULARS TEXT", 146, 300)], LAYOUT, V))
    assert got == {"type": ["NET BANKING"], "description": ["PARTICULARS TEXT"]}


def test_a_right_aligned_figure_under_a_left_aligned_label_goes_to_the_last_column_before_it():
    layout = Layout([Column("description", 109, 153), Column("debit", 339, 359), Column("credit", 409, 432),
                     Column("balance", 479, 510)])
    got = texts(assign([Chunk("4,587.89", 370, 401), Chunk("9,376.75", 520, 551)], layout, V))
    assert got == {"debit": ["4,587.89"], "balance": ["9,376.75"]}


def test_a_foreign_amount_with_its_currency_is_a_figure_for_the_foreign_column():
    layout = Layout([Column("description", 160, 240), Column("foreign_amount", 436, 480), Column("amount", 520, 557)])
    got = texts(assign([Chunk("USD 25.00", 436, 480), Chunk("2,075.00", 520, 557)], layout, V))
    assert got == {"foreign_amount": ["USD 25.00"], "amount": ["2,075.00"]}


def test_a_date_glued_to_the_text_after_it_is_split_off():
    chunks = split_leading_date([Chunk("01 Aug 26 PAYMENT RECEIVED", 26, 126), Chunk("10.00", 300, 330)], V)
    assert [c.text for c in chunks] == ["01 Aug 26", "PAYMENT RECEIVED", "10.00"]
    assert chunks[0].x0 == 26 and chunks[1].x1 == 126 and chunks[0].x1 < chunks[1].x0 + 1


def test_a_chunk_that_is_only_a_date_or_does_not_start_with_one_is_left_alone():
    one = [Chunk("01/08/2026", 26, 66)]
    assert split_leading_date(one, V) == one
    other = [Chunk("PAYMENT 01/08/2026", 26, 126)]
    assert split_leading_date(other, V) == other
