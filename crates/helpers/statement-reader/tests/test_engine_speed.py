"""A ten-page statement reads in well under two seconds."""

import time
from datetime import date

from statement_reader.document import Document
from statement_reader.engine import read_document
from statement_reader.read.pdf_words import open_pdf

from .synthetic.data import indian, ledger
from .synthetic.draw import Pdf


def ten_page_pdf(path):
    rows = ledger(31, 200, 5_000_000, date(2026, 2, 1))
    pdf = Pdf()
    for page in range(10):
        pdf.new_page()
        for x, label, right in [(40, "Date", False), (110, "Description", False), (400, "Debit", True),
                                (470, "Credit", True), (550, "Balance", True)]:
            pdf.text(x, 60, label, bold=True, right=right)
        for i, row in enumerate(rows[page * 20:(page + 1) * 20]):
            y = 80 + 14 * i
            pdf.text(40, y, row.date.strftime("%d/%m/%Y"))
            pdf.text(110, y, row.description)
            if row.debit:
                pdf.text(400, y, indian(row.debit), right=True)
            if row.credit:
                pdf.text(470, y, indian(row.credit), right=True)
            pdf.text(550, y, indian(row.balance), right=True)
    pdf.save(path)
    return rows


def test_ten_pages_and_two_hundred_rows_read_exactly_and_fast(tmp_path):
    path = tmp_path / "ten.pdf"
    rows = ten_page_pdf(path)
    reader = open_pdf(str(path))
    started = time.perf_counter()
    result = read_document(Document("pdf", pages=reader))
    seconds = time.perf_counter() - started
    assert result.outcome == "read" and len(result.statements[0].transactions) == 200
    assert [t.amount_paise for t in result.statements[0].transactions] == [r.debit or r.credit for r in rows]
    assert result.statements[0].checks[0].outcome == "passed"
    assert seconds < 2.0
