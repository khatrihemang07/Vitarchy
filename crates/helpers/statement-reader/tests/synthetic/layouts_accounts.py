"""PDFs holding several accounts or card sections (traits f, g)."""

from __future__ import annotations

from datetime import date
from pathlib import Path

from .built import Built, expected_statements, statement
from .data import Row, indian, ledger, plain, totals
from .draw import Pdf

ISSUER = "Example Bank"
CONSOLIDATED_COLUMNS = [("Date", 40, False), ("Transaction Reference", 100, False),
                        ("Ref.No./Chq.No.", 300, False), ("Credit", 440, True),
                        ("Debit", 500, True), ("Balance", 560, True)]


def _consolidated_header(pdf: Pdf, y: float) -> None:
    for label, x, right in CONSOLIDATED_COLUMNS:
        pdf.text(x, y, label, bold=True, right=right)


def _consolidated_rows(pdf: Pdf, rows: list[Row], y: float) -> float:
    for row in rows:
        pdf.text(40, y, row.date.strftime("%d-%m-%y"))
        pdf.text(100, y, row.description[:50])
        pdf.text(300, y, "-")
        pdf.text(440, y, plain(row.credit), right=True)
        pdf.text(500, y, plain(row.debit), right=True)
        pdf.text(560, y, plain(row.balance), right=True)
        row.description = row.description[:50]
        y += 16
    return y


def consolidated(out: Path) -> Built:
    """(f) A savings section across two pages (header repeated, no label) and a loan section."""
    savings = ledger(7, 20, 5_000_000, date(2026, 8, 1))
    loan = ledger(8, 2, 22_271_900, date(2026, 8, 5), inverted=True)
    pdf = Pdf()
    pdf.new_page()
    pdf.text(40, 36, ISSUER)
    pdf.text(40, 50, "Relationship Summary", 12, bold=True)
    pdf.text(40, 64, "As on 31-08-26")
    pdf.text(40, 90, "TRANSACTION DETAILS", bold=True)
    pdf.text(40, 104, "SAVING ACCOUNT")
    pdf.text(140, 104, "xxxxxxx1111")
    pdf.text(40, 122, "TRANSACTION OVERVIEW", bold=True)
    pdf.text(40, 136, "Your Opening Balance on 01-08-26:")
    pdf.text(240, 136, plain(5_000_000))
    for x in range(6):
        pdf.text(40 + x * 40, 148, "null", invisible=True)
    _consolidated_header(pdf, 160)
    _consolidated_rows(pdf, savings[:12], 178)
    pdf.new_page()
    _consolidated_header(pdf, 60)
    _consolidated_rows(pdf, savings[12:], 78)
    pdf.text(40, 800, "*All dates are in DD-MM-YY format")
    pdf.new_page()
    pdf.text(40, 90, "DL/TL ACCOUNT", bold=True)
    pdf.text(140, 90, "xxxxxxx2222")
    pdf.text(40, 110, "Your Opening Balance on 01-08-26:")
    pdf.text(240, 110, plain(22_271_900))
    _consolidated_header(pdf, 130)
    y = _consolidated_rows(pdf, loan, 148)
    pdf.text(40, y + 10, "Your Closing Balance on 31-08-26:")
    pdf.text(240, y + 10, plain(loan[-1].balance))
    pdf.save(out / "consolidated.pdf")
    period = (date(2026, 8, 1), date(2026, 8, 31))
    first = statement(ISSUER, "savings", "xxxxxxx1111", period, savings, opening_paise=5_000_000)
    second = statement(ISSUER, "loan", "xxxxxxx2222", period, loan, opening_paise=22_271_900,
                       closing_paise=loan[-1].balance)
    return Built("consolidated.pdf", expected_statements([first, second]),
                 {"format": "pdf", "pages": 3, "words": pdf.words, "invisible_null": 6})


def card_with_addon(out: Path) -> Built:
    """(g) Card sections for a primary and an add-on card, and an international row."""
    rows = ledger(9, 9, 0, date(2026, 7, 2))
    previous = 5_000_000
    pdf = Pdf()
    pdf.new_page()
    pdf.text(40, 60, ISSUER, 12, bold=True)
    pdf.text(40, 76, "CREDIT CARD STATEMENT")
    pdf.text(40, 90, "Card Number 4000XXXXXXXX1111")
    pdf.text(40, 102, "Statement period : June 29, 2026 to July 28, 2026")
    heads = [(40, "Date", False), (100, "SerNo.", False), (160, "Transaction Details", False),
             (400, "Reward", False), (436, "Intl.#", False), (545, "Amount (in Rs.)", True)]
    for x, label, right in heads:
        pdf.text(x, 200, label, bold=True, right=right)
    pdf.text(400, 209, "Points", bold=True)
    pdf.text(436, 209, "amount", bold=True)
    extras, y = {}, 228
    for i, row in enumerate(rows):
        if i in (0, 5):
            mask = "4000XXXXXXXX1111" if i == 0 else "4000XXXXXXXX2222"
            pdf.text(40, y, mask)
            y += 16
        row.description = row.description[:40]
        row.balance = None
        extra = {"source_id": f"1400{i:07d}"}
        if i >= 5:
            extra["addon_card"] = "4000XXXXXXXX2222"
        if i == 7:
            row.credit, row.debit = 0, 207_500
            extra |= {"foreign_amount": "25.00", "foreign_currency": "USD"}
            pdf.text(436, y, "USD 25.00")
        extras[i] = extra
        pdf.text(40, y, row.date.strftime("%d/%m/%Y"))
        pdf.text(100, y, extra["source_id"])
        pdf.text(160, y, row.description)
        pdf.text(420, y, "-20" if row.credit else "10")
        pdf.text(540, y, indian(row.debit or row.credit), right=True)
        if row.credit:
            pdf.text(548, y, "CR")
        y += 14
    debit, credit, _, _ = totals(rows)
    due = previous + debit - credit
    for i, (label, value) in enumerate(zip(["Previous Balance", "Purchases", "Payments",
                                            "Total Amount Due"],
                                           [previous, debit, credit, due], strict=True)):
        pdf.text(40 + i * 120, 140, label, 7)
        pdf.text(40 + i * 120, 156, indian(value))
    pdf.text(40, y + 12, "# International Spends")
    pdf.save(out / "card_with_addon.pdf")
    stmt = statement(ISSUER, "credit_card", "4000XXXXXXXX1111",
                     (date(2026, 6, 29), date(2026, 7, 28)), rows, extras, opening_paise=previous,
                     closing_paise=due, total_debit_paise=debit, total_credit_paise=credit)
    return Built("card_with_addon.pdf", expected_statements([stmt]),
                 {"format": "pdf", "pages": 1, "words": pdf.words})
