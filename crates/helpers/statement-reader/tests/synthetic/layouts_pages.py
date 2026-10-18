"""Multi-page, headerless and vertically-centred PDF layouts (traits d, e, h)."""

from __future__ import annotations

from datetime import date
from pathlib import Path

from .built import Built, expected_statements, statement
from .data import indian, ledger, plain
from .draw import Pdf, hard_wrap, wrap_words

ISSUER = "Example Bank"
LEGEND = [
    "Legends for transactions in your Account Statement",
    "RCHG - Recharge", "BIL - Internet Bill payment", "INFT - Internal Fund Transfer",
    "This is a system generated statement. Hence, it does not require any signature.",
]


def two_line_header_pages(out: Path) -> Built:
    """(d) Two-line header repeated per page, page Total: rows, a B/F row and a trailing legend."""
    rows = ledger(4, 24, 30_000_000, date(2026, 3, 2))
    opening = 30_000_000
    pdf = Pdf()
    xs = {"sno": 40, "date": 70, "remarks": 140, "wd": 440, "dep": 500, "bal": 560}
    per_page, index = 8, 0
    for page_no in range(3):
        pdf.new_page()
        pdf.text(300, 40, str(page_no + 1))
        if page_no == 0:
            pdf.text(40, 60, ISSUER, 12, bold=True)
            pdf.text(40, 76, "Statement of Transactions in Savings Account XXXXXX0234 in INR "
                             "for the period March 01, 2026 - April 30, 2026")
        top = 100
        pdf.rect(30, top - 12, 570, top + 20, fill=(0.85, 0.85, 0.85))
        pdf.text(xs["sno"], top, "S No.", bold=True)
        pdf.text(xs["date"], top, "Transaction", bold=True)
        pdf.text(xs["date"], top + 10, "Date", bold=True)
        pdf.text(xs["remarks"], top, "Transaction Remarks", bold=True)
        for key, upper in (("wd", "Withdrawal"), ("dep", "Deposit")):
            pdf.text(xs[key], top, upper, bold=True, right=True)
            pdf.text(xs[key], top + 10, "Amount (INR)", bold=True, right=True)
        pdf.text(xs["bal"], top, "Balance", bold=True, right=True)
        pdf.text(xs["bal"], top + 10, "(INR)", bold=True, right=True)
        y = top + 36
        if page_no == 0:
            pdf.text(xs["date"], y, "01.03.2026")
            pdf.text(xs["remarks"], y, "B/F")
            pdf.text(xs["bal"], y, indian(opening), right=True)
            y += 14
        wd = dep = 0
        for row in rows[index:index + per_page]:
            index += 1
            pdf.text(xs["sno"], y, str(index))
            pdf.text(xs["date"], y, row.date.strftime("%d.%m.%Y"))
            pdf.text(xs["remarks"], y, row.description)
            if row.debit:
                pdf.text(xs["wd"], y, plain(row.debit), right=True)
                wd += row.debit
            if row.credit:
                pdf.text(xs["dep"], y, plain(row.credit), right=True)
                dep += row.credit
            pdf.text(xs["bal"], y, plain(row.balance), right=True)
            y += 14
        pdf.text(xs["remarks"], y, "Total:", bold=True)
        pdf.text(xs["wd"], y, plain(wd), right=True)
        pdf.text(xs["dep"], y, plain(dep), right=True)
        pdf.text(xs["bal"], y, plain(rows[index - 1].balance), right=True)
        if page_no == 2:
            for i, line in enumerate(LEGEND):
                pdf.text(40, y + 30 + i * 11, line)
    pdf.save(out / "two_line_header_pages.pdf")
    stmt = statement(ISSUER, "savings", "XXXXXX0234", (date(2026, 3, 1), date(2026, 4, 30)),
                     rows, opening_paise=opening)
    return Built("two_line_header_pages.pdf", expected_statements([stmt]),
                 {"format": "pdf", "pages": 3, "words": pdf.words})


def header_blobs(pdf: Pdf, labels: list[tuple[float, str]], top: float, bottom: float) -> None:
    """Draw header labels as white rectangles per letter: a vector 'outline' with no text layer."""
    pdf.rect(40, top, 555, bottom, fill=(0.25, 0.1, 0.45))
    for x, label in labels:
        for i, ch in enumerate(label):
            if ch != " ":
                pdf.rect(x + i * 5, top + 8, x + i * 5 + 3.5, top + 16, fill=(1, 1, 1))


def headerless(out: Path) -> Built:
    """(e) Header drawn as vector paths only; Date | Description | Debit | Credit | Balance."""
    rows = ledger(5, 10, 8_000_000, date(2026, 6, 2))
    pdf = Pdf()
    pdf.new_page()
    pdf.text(40, 60, ISSUER, 12, bold=True)
    pdf.text(40, 76, "STATEMENT OF ACCOUNT")
    pdf.text(40, 90, "Account Number : XXXXXX0456")
    pdf.text(40, 102, "Statement From : 01/06/2026 to 30/06/2026")
    edges = [40, 110, 330, 400, 470, 555]
    top, height = 120, 26
    header_blobs(pdf, [(edges[0] + 4, "Date"), (edges[1] + 4, "Description"),
                       (edges[2] + 4, "Debit"), (edges[3] + 4, "Credit"),
                       (edges[4] + 4, "Balance")], top, top + 24)
    pdf.text(edges[1] + 4, top + 34, "null", invisible=True)
    for r in range(len(rows) + 1):
        pdf.hline(edges[0], edges[-1], top + 24 + r * height)
    for x in edges:
        pdf.vline(x, top, top + 24 + len(rows) * height)
    for r, row in enumerate(rows):
        row.description = f"{row.description} PAYMENT REF {7100 + r} MONTHLY SETTLEMENT"
        lines = wrap_words(row.description, 205)
        base = top + 24 + r * height + 11
        pdf.text(edges[0] + 4, base, row.date.strftime("%d/%m/%Y"))
        for k, line in enumerate(lines[:2]):
            pdf.text(edges[1] + 4, base + 9 * k, line)
        pdf.text(edges[3] - 4, base, plain(row.debit) if row.debit else "-", right=True)
        pdf.text(edges[4] - 4, base, plain(row.credit) if row.credit else "-", right=True)
        pdf.text(edges[5] - 4, base, plain(row.balance), right=True)
        row.description = " ".join(lines[:2])
    pdf.save(out / "headerless.pdf")
    stmt = statement(ISSUER, "savings", "XXXXXX0456", (date(2026, 6, 1), date(2026, 6, 30)), rows)
    return Built("headerless.pdf", expected_statements([stmt]),
                 {"format": "pdf", "pages": 1, "words": pdf.words, "invisible_null": 1})


def centred_narration(out: Path) -> Built:
    """(h) Narration vertically centred in its row: its first line sits above the date line."""
    rows = ledger(6, 7, 10_000_000, date(2026, 8, 3))
    pdf = Pdf()
    pdf.new_page()
    pdf.text(37, 60, ISSUER, 12, bold=True)
    pdf.text(37, 76, "Statement of Transactions in Savings Account XXXXXX8765 in INR for the "
                     "period August 01, 2026 - August 31, 2026")
    pdf.rect(30, 90, 565, 108, fill=(0.88, 0.88, 0.88))
    right = {"DEPOSITS": 407, "WITHDRAWALS": 486, "BALANCE": 559}
    pdf.text(37, 102, "DATE", bold=True)
    pdf.text(77, 102, "MODE", bold=True)
    pdf.text(144, 102, "PARTICULARS", bold=True)
    for label, x in right.items():
        pdf.text(x, 102, label, bold=True, right=True)
    y = 118  # baseline of the first narration line
    pdf.text(37, y, "01-08-2026")
    pdf.text(144, y, "B/F")
    pdf.text(559, y, indian(10_000_000), right=True)
    y += 9
    extras = {}
    for i, row in enumerate(rows):
        payee = f"FAKE PERSON {i + 1}"
        upi = f"UPI/{600000000000 + i * 13}/NA/fake{i}@exb/Example Bank/EXB{i}c3c5d0a1b2e3f4a5"
        tail = hard_wrap(upi, 110)
        lines = [payee, *tail[: 1 + i % 3]]
        mid = len(lines) // 2  # the date/amount line is in the middle, below the first line
        row.description = payee
        for k, piece in enumerate(lines[1:], start=1):
            moded = i in (2, 4) and k == mid  # the MODE cell is printed on the date line: keep it
            row.description += (" " if k == 1 or moded else "") + ("NET BANKING " if moded else "") + piece
        for k, line in enumerate(lines):
            pdf.text(144, y + 9 * k, line, bold=(k == 0))
        base = y + 9 * mid
        pdf.text(37, base, row.date.strftime("%d-%m-%Y"))
        if i in (2, 4):
            pdf.text(92, base, "NET BANKING")
        if row.credit:
            pdf.text(407, base, indian(row.credit), right=True)
        if row.debit:
            pdf.text(486, base, indian(row.debit), right=True)
        pdf.text(559, base, indian(row.balance), right=True)
        y += 9 * len(lines)
    pdf.save(out / "centred_narration.pdf")
    stmt = statement(ISSUER, "savings", "XXXXXX8765", (date(2026, 8, 1), date(2026, 8, 31)),
                     rows, extras, opening_paise=10_000_000)
    return Built("centred_narration.pdf", expected_statements([stmt]),
                 {"format": "pdf", "pages": 1, "words": pdf.words, "bold_lines": len(rows)})



