from statement_reader.contract import Summary
from statement_reader.locate.grid_header import find_grid_header
from statement_reader.normalize.payments import (
    instrument_issuer,
    instrument_label,
    payment_history_statements,
)
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


def _row(instrument, when="Sept 19, 2026", clock="11:20 am", **cells):
    return RawRow({"date": when, "time": clock, "amount": "650", "type": "CREDIT", "source_id": "T1",
                   "utr": "512345678952", "instrument": instrument, **cells}, "Paid to X", 0)


def test_a_row_becomes_a_transaction_with_date_time_utr_and_app_id():
    (section,) = payment_history_statements([_row("Paid by CARD R")], V)
    (txn,) = section.transactions
    assert (txn.date, txn.time, txn.amount_paise, txn.direction) == ("2026-09-19", "11:20:00", 65000, "credit")
    assert (txn.reference, txn.source_id, txn.balance_paise, txn.description) == ("512345678952", "T1", None, "Paid to X")


def test_a_row_without_a_time_column_has_no_time():
    (section,) = payment_history_statements([_row("Paid by CARD R", clock="")], V)
    assert section.transactions[0].time is None


def test_one_section_per_instrument_in_first_seen_order_with_period_and_no_checks():
    rows = [_row("Paid by ICICI R", "Oct 02, 2026"), _row("Paid by XXXXXX4321", "Oct 01, 2026"),
            _row("Credited to ICICI R", "Sept 20, 2026"), _row("", "Sept 21, 2026")]
    sections = payment_history_statements(rows, V)
    assert [s.account.mask for s in sections] == ["ICICI R", "XXXXXX4321", None]
    assert [s.account.issuer for s in sections] == ["ICICI Bank", None, None]
    assert all(s.account.kind is None and s.checks == [] for s in sections)
    assert [len(s.transactions) for s in sections] == [2, 1, 1]
    assert (sections[0].period.start, sections[0].period.end) == ("2026-09-20", "2026-10-02")
    assert sections[0].summary == Summary()


def test_the_instrument_issuer_comes_from_the_vocabulary():
    assert instrument_issuer("ICICI R", V) == "ICICI Bank"
    assert instrument_issuer("BOB R", V) == "Bank of Baroda"
    assert instrument_issuer("XXXXXX4321", V) is None
    assert instrument_issuer("Wallet", V) is None
    assert instrument_issuer(None, V) is None
