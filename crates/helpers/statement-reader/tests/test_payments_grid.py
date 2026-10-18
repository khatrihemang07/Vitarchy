from statement_reader.locate.grid_header import find_grid_header
from statement_reader.normalize.payments import build_payment_records, instrument_details, instrument_label
from statement_reader.segment.grid_rows import grid_rows
from statement_reader.segment.rows import RawRow
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()
HEADER = ["Date", "Details", "Debit", "Credit", "Balance"]


def test_a_header_row_is_found_below_a_preamble_and_a_two_row_header_is_merged():
    grid = [["Bank", "", "", "", ""], ["Account", "X1", "", "", ""], HEADER,
            ["01/03/2026", "A", "5.00", "", "95.00"]]
    found = find_grid_header(grid, V)
    assert (found.first, found.last) == (2, 2)
    two = [["Date", "Details", "Amount", "", "Balance"], ["", "", "Debit", "Credit", ""],
           ["01/03/2026", "A", "5.00", "", "95.00"]]
    found = find_grid_header(two, V)
    assert (found.first, found.last) == (0, 1)
    assert [c.field for c in found.layout.columns] == ["date", "description", "debit", "credit", "balance"]


def test_no_header_in_a_grid_without_date_and_money_columns():
    assert find_grid_header([["a", "b", "c"], ["1", "2", "3"]], V) is None


def test_rows_run_until_the_date_stops_parsing_with_blank_and_spill_rows_tolerated():
    grid = [HEADER, ["01/03/2026", "UPI/1/part one", "5.00", "", "95.00"], ["", "part two", "", "", ""], [""] * 5,
            ["02/03/2026", "B", "", "1.00", "96.00"], [""] * 5, ["Legends", "", "", "", ""],
            ["03/03/2026", "late", "1.00", "", "95.00"]]
    header = find_grid_header(grid, V)
    rows = grid_rows(grid, header.last + 1, header.layout, V)
    assert [r.description for r in rows] == ["UPI/1/part one part two", "B"]
    assert rows[0].cells["debit"] == "5.00"


def test_instrument_prefixes_come_from_the_vocabulary():
    assert instrument_label("Paid by XXXXXX4321", V) == "XXXXXX4321"
    assert instrument_label("Credited to ICICI R", V) == "ICICI R"
    assert instrument_label("", V) is None


def test_payment_records_combine_date_and_time_and_take_direction_from_the_type_column():
    row = RawRow({"date": "Sept 19, 2026", "time": "11:20 am", "amount": "650", "type": "CREDIT",
                  "source_id": "T1", "utr": "512345678952", "instrument": "Paid by CARD R"},
                 "Paid to X", 0)
    (record,) = build_payment_records([row], V)
    assert (record.occurred_at, record.amount_paise, record.direction) == ("2026-09-19T11:20:00", 65000, "credit")
    assert (record.utr, record.instrument, record.source_id) == ("512345678952", "CARD R", "T1")


def test_the_instrument_issuer_and_trailing_digits_come_from_the_vocabulary():
    assert instrument_details("ICICI R", V) == ("ICICI Bank", None)
    assert instrument_details("BOB R", V) == ("Bank of Baroda", None)
    assert instrument_details("XXXXXX4321", V) == (None, "4321")
    assert instrument_details("XXXXXXXXXX56", V) == (None, "56")
    assert instrument_details("Wallet", V) == (None, None)
    assert instrument_details(None, V) == (None, None)
