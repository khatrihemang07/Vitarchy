"""Layouts unlike any real sample (traits m, n, o): the engine must read them with vocabulary only."""

from __future__ import annotations

import csv
from datetime import date
from pathlib import Path

from .built import Built, expected_statements, statement
from .data import indian, ledger, plain
from .draw import Pdf

ISSUER = "Example Bank"


def balance_first(out: Path) -> Built:
    """(m) Balance column FIRST, long month names, and header words the vocabulary learns from this file."""
    rows = ledger(21, 9, 4_000_000, date(2026, 3, 3))
    pdf = Pdf()
    pdf.new_page()
    pdf.text(40, 60, ISSUER, 12, bold=True)
    pdf.text(40, 76, "SAVINGS ACCOUNT STATEMENT")
    pdf.text(40, 90, "Account Number : XXXXXX0909")
    pdf.text(40, 102, "Statement Period : 01/03/2026 to 31/03/2026")
    for x, label, right in [(110, "Running Bal", True), (130, "Posted On", False),
                            (250, "Narrative", False), (480, "Outflow", True), (560, "Inflow", True)]:
        pdf.text(x, 140, label, bold=True, right=right)
    for i, row in enumerate(rows):
        y = 160 + 16 * i
        pdf.text(110, y, indian(row.balance), right=True)
        pdf.text(130, y, f"{row.date.day:02d} March 2026")
        pdf.text(250, y, row.description[:40])
        row.description = row.description[:40]
        if row.debit:
            pdf.text(480, y, indian(row.debit), right=True)
        if row.credit:
            pdf.text(560, y, indian(row.credit), right=True)
    pdf.save(out / "balance_first.pdf")
    stmt = statement(ISSUER, "savings", "XXXXXX0909", (date(2026, 3, 1), date(2026, 3, 31)), rows)
    return Built("balance_first.pdf", expected_statements([stmt]), {"format": "pdf", "pages": 1, "words": pdf.words})


def signed_amount(out: Path) -> Built:
    """(n) One signed Amount column (minus = money out), dd-Mon-yyyy dates, no rules."""
    rows = ledger(22, 10, 2_500_000, date(2026, 5, 4))
    pdf = Pdf()
    pdf.new_page()
    pdf.text(40, 60, ISSUER, 12, bold=True)
    pdf.text(40, 76, "Statement of Account")
    pdf.text(40, 90, "Account No : XXXXXX0910")
    pdf.text(40, 102, "Period : 01-May-2026 to 31-May-2026")
    for x, label, right in [(40, "Date", False), (120, "Particulars", False), (440, "Amount", True),
                            (540, "Balance", True)]:
        pdf.text(x, 140, label, bold=True, right=right)
    for i, row in enumerate(rows):
        y = 160 + 16 * i
        pdf.text(40, y, f"{row.date.day:02d}-May-2026")
        pdf.text(120, y, row.description[:45])
        row.description = row.description[:45]
        signed = f"-{indian(row.debit)}" if row.debit else indian(row.credit)
        pdf.text(440, y, signed, right=True)
        pdf.text(540, y, indian(row.balance), right=True)
    pdf.save(out / "signed_amount.pdf")
    stmt = statement(ISSUER, "savings", "XXXXXX0910", (date(2026, 5, 1), date(2026, 5, 31)), rows)
    return Built("signed_amount.pdf", expected_statements([stmt]), {"format": "pdf", "pages": 1, "words": pdf.words})


def month_first_csv(out: Path) -> Built:
    """(o) A CSV: month-first dates, a Dr/Cr type column beside one Amount column, a Closing Balance."""
    rows = ledger(23, 8, 1_000_000, date(2026, 8, 13))
    path = out / "month_first.csv"
    with open(path, "w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle, lineterminator="\n")
        writer.writerow([f"{ISSUER} - Account XXXXXX0911"])
        writer.writerow(["Statement Period", "08/01/2026 to 08/31/2026"])
        writer.writerow([])
        writer.writerow(["Posting Date", "Remarks", "Amount", "Dr/Cr", "Closing Balance"])
        for row in rows:
            writer.writerow([row.date.strftime("%m/%d/%Y"), row.description, plain(row.debit or row.credit),
                             "Dr" if row.debit else "Cr", plain(row.balance)])
    stmt = statement(ISSUER, "savings", "XXXXXX0911", (date(2026, 8, 1), date(2026, 8, 31)), rows)
    return Built(path.name, expected_statements([stmt]), {"format": "csv", "rows": len(rows) + 4, "cols": 5})
