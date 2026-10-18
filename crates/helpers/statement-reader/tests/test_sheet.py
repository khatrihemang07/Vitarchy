import csv
from datetime import datetime

import openpyxl

from statement_reader.read.sheet import decode_text, pick_delimiter, read_csv, read_grid, read_xlsx

from .conftest import FILES, MANIFEST


def grid_of(name):
    return read_grid(str(FILES / name), MANIFEST[name]["format"])


def test_the_payment_history_csv_keeps_its_preamble_and_footer_and_strips_the_bom():
    grid = grid_of("payment_history.csv")
    facts = MANIFEST["payment_history.csv"]
    assert (len(grid), len(grid[0])) == (facts["rows"], facts["cols"])
    assert grid[0][0] == "Transaction Statement for 9000000000"
    assert grid[3][:3] == ["Date", "Time", "Transaction Details"]
    assert grid[4][0] == "Oct 18, 2026" and grid[4][1] == "11:20 am" and grid[4][7] == "650"
    assert any(row[0].startswith("Sept ") for row in grid)


def test_the_encrypted_workbook_grid_after_unlocking_has_merged_headers_once():
    from statement_reader.open.sniff import sniff
    from statement_reader.open.unlock import unlock

    path = str(FILES / "encrypted_sheet.xlsx")
    unlocked = unlock(path, sniff(path), ["xlsx2026"], None)
    grid = read_xlsx(unlocked.source)
    facts = MANIFEST["encrypted_sheet.xlsx"]
    assert (len(grid), len(grid[0])) == (facts["rows"], facts["cols"])
    assert grid[1][1] == "Example Bank, Sample Branch" and grid[1][2:] == ["", "", "", ""]
    assert grid[6] == ["Date", "Details", "Ref No", "Amount", "", "Balance"]
    assert grid[7] == ["", "", "", "Debit", "Credit", ""]
    first = grid[facts["first_data_row"] - 1]
    assert first[0].startswith("2026-09-")
    assert len(first[0]) == 10  # a real datetime cell becomes an ISO date
    assert first[5].count(".") == 1 and len(first[5].split(".")[1]) == 2
    assert grid[-3][0] == "Statement Summary : 01-09-2026 To 30-09-2026"
    assert grid[-2][0] == "Brought Forward"


def test_the_legacy_workbook_grid_has_spacer_columns_a_ghost_row_and_merged_preamble():
    grid = grid_of("ghost_row.xls")
    facts = MANIFEST["ghost_row.xls"]
    assert (len(grid), len(grid[0])) == (facts["rows"], facts["cols"])
    assert grid[3][3] == "XXXXXX0555 ( INR )  - FAKE NAME" and grid[3][4:] == [""] * 5
    assert all(row[0] == "" and row[9 - 1] is not None for row in grid)  # spacer column A
    header = next(row for row in grid if row[1] == "S No.")
    assert header[2:] == ["Value Date", "Transaction Date", "Cheque Number", "Transaction Remarks",
                          "Withdrawal Amount(INR)", "Deposit Amount(INR)", "Balance(INR)"]
    ghosts = [row for row in grid if row[1] == "" and row[5] and not row[2]]
    assert len(ghosts) == facts["ghost_rows"]
    assert all(not any(c for i, c in enumerate(ghost) if i != 5) for ghost in ghosts)


def test_an_xlsx_real_date_cell_is_iso_and_a_datetime_keeps_its_time(tmp_path):
    book = openpyxl.Workbook()
    book.active["A1"] = datetime(2026, 1, 2)
    book.active["B1"] = datetime(2026, 1, 2, 9, 30)
    book.active["C1"] = 5500.5
    book.active["C1"].number_format = "0.00"
    book.active["D1"] = True
    path = tmp_path / "x.xlsx"
    book.save(path)
    assert read_xlsx(str(path)) == [["2026-01-02", "2026-01-02T09:30:00", "5500.50", "TRUE"]]


def test_an_empty_workbook_is_an_empty_grid(tmp_path):
    path = tmp_path / "x.xlsx"
    openpyxl.Workbook().save(path)
    assert read_xlsx(str(path)) == []


def write(path, data: bytes):
    path.write_bytes(data)
    return str(path)


def test_csv_without_bom_and_with_quotes_and_embedded_delimiters(tmp_path):
    path = write(tmp_path / "a.csv", b'Name,Note\n"Smith, J","said ""hi"""\nplain,"two\nlines"\n')
    assert read_csv(path) == [["Name", "Note"], ["Smith, J", 'said "hi"'], ["plain", "two\nlines"]]


def test_csv_semicolon_and_tab_delimiters_are_sniffed(tmp_path):
    assert read_csv(write(tmp_path / "s.csv", b"a;b;c\n1;2;3\n"))[1] == ["1", "2", "3"]
    assert read_csv(write(tmp_path / "t.csv", b"a\tb\tc\n1\t2\t3\n"))[1] == ["1", "2", "3"]


def test_csv_ragged_rows_are_padded(tmp_path):
    grid = read_csv(write(tmp_path / "r.csv", b"title\n\na,b,c\n1,2,3\nfooter\n"))
    assert {len(r) for r in grid} == {3} and grid[0] == ["title", "", ""]


def test_a_delimiter_inside_a_preamble_line_does_not_win():
    rows = {",": [["a", "b", "c"]] * 5 + [["x", "y"]], ";": [["a,b,c"]] * 5 + [["x", "y"]]}
    assert pick_delimiter(rows) == ","


def test_text_decoding_handles_boms_utf16_and_legacy_bytes():
    assert decode_text("﻿hi".encode()) == "hi"
    assert decode_text("hi".encode("utf-16")) == "hi"
    assert decode_text(b"caf\xe9") == "café"


def test_csv_written_by_the_csv_module_round_trips(tmp_path):
    path = tmp_path / "q.csv"
    with open(path, "w", newline="") as handle:
        csv.writer(handle, quoting=csv.QUOTE_ALL).writerows([["a", "b"], ["1", "x,y"]])
    assert read_csv(str(path)) == [["a", "b"], ["1", "x,y"]]
