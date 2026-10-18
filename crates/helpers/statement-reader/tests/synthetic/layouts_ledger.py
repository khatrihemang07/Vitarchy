"""Single-table PDF layouts: ruled, borderless-and-wrapped, and side-panel (traits a, b, c, i)."""

from __future__ import annotations

from datetime import date
from pathlib import Path

from .built import Built, expected_statements, statement
from .data import indian, ledger, totals
from .draw import Pdf, wrap_words

ISSUER = "Example Bank"


def ruled_simple(out: Path, name: str = "ruled_simple.pdf", seed: int = 1,
                 encryption: dict | None = None) -> Built:
    """(a) A fully ruled table: Date | Description | Debit | Credit | Balance."""
    rows = ledger(seed, 12, 500_000, date(2026, 3, 2))
    pdf = Pdf()
    pdf.new_page()
    pdf.text(40, 60, ISSUER, 12, bold=True)
    pdf.text(40, 76, "Statement of Account")
    pdf.text(40, 90, "Account Number : XXXXXX0123")
    pdf.text(40, 102, "Statement From : 01/03/2026 to 31/03/2026")
    edges = [40, 105, 335, 405, 475, 555]
    top, height = 120, 18
    for i, label in enumerate(["Date", "Description", "Debit", "Credit", "Balance"]):
        pdf.text(edges[i] + 4, top + 12, label, bold=True)
    for r in range(len(rows) + 2):
        pdf.hline(edges[0], edges[-1], top + r * height)
    for x in edges:
        pdf.vline(x, top, top + (len(rows) + 1) * height)
    for r, row in enumerate(rows):
        base = top + (r + 1) * height + 12
        pdf.text(edges[0] + 4, base, row.date.strftime("%d/%m/%Y"))
        pdf.text(edges[1] + 4, base, row.description)
        if row.debit:
            pdf.text(edges[3] - 4, base, indian(row.debit), right=True)
        if row.credit:
            pdf.text(edges[4] - 4, base, indian(row.credit), right=True)
        pdf.text(edges[5] - 4, base, indian(row.balance), right=True)
    pdf.save(out / name, **(encryption or {}))
    stmt = statement(ISSUER, "savings", "XXXXXX0123", (date(2026, 3, 1), date(2026, 3, 31)), rows)
    return Built(name, expected_statements([stmt]),
                 {"format": "pdf", "pages": 1, "words": pdf.words, "ruled": True})


LONG_PARTS = [
    "PAYMENT TO {m} FOR ORDER NUMBER {n} INCLUDING DELIVERY AND HANDLING CHARGES",
    "{m} ONLINE PURCHASE REF {n} MEMBERSHIP RENEWAL ANNUAL PLAN",
    "{m} {n}",
]


def borderless_wrapped(out: Path) -> Built:
    """(b) No rules; descriptions wrap to 2-3 lines; Indian grouping; CR suffix as its own token."""
    rows = ledger(2, 14, 0, date(2026, 4, 3), big_at=5)
    pdf = Pdf()
    pdf.new_page()
    pdf.text(40, 60, ISSUER, 12, bold=True)
    pdf.text(40, 76, "CREDIT CARD STATEMENT")
    pdf.text(40, 90, "Card Number 4000XXXXXXXX7777")
    pdf.text(40, 102, "Statement Period : April 01, 2026 to April 30, 2026")
    for x, label in [(40, "Date"), (112, "SerNo."), (170, "Transaction Details")]:
        pdf.text(x, 140, label, bold=True)
    pdf.text(520, 140, "Amount (in Rs.)", bold=True, right=True)
    y, extras = 160, {}
    for i, row in enumerate(rows):
        row.description = LONG_PARTS[i % 3].format(m=row.description.split("/")[-1], n=100000 + i)
        serial = f"1368{7000 + i * 7:07d}"
        extras[i] = {"source_id": serial, "balance_paise": None}
        lines = wrap_words(row.description, 190)
        pdf.text(40, y, row.date.strftime("%d/%m/%Y"))
        pdf.text(112, y, serial)
        for k, line in enumerate(lines):
            pdf.text(170, y + 9 * k, line)
        amount = indian(row.debit or row.credit)
        pdf.text(520, y, amount, right=True)
        if row.credit:
            pdf.text(526, y, "CR")
        y += 9 * len(lines) + 10
        row.description = " ".join(row.description.split())
    debit, credit, _, _ = totals(rows)
    pdf.text(40, y + 10, "# International Spends")
    pdf.text(580, 60, "42832/50000", rotate=90)
    pdf.save(out / "borderless_wrapped.pdf")
    stmt = statement(ISSUER, "credit_card", "4000XXXXXXXX7777",
                     (date(2026, 4, 1), date(2026, 4, 30)), rows, extras)
    return Built("borderless_wrapped.pdf", expected_statements([stmt]),
                 {"format": "pdf", "pages": 1, "words": pdf.words, "rotated_dropped": 1})


def side_panel(out: Path) -> Built:
    """(c) Junk in the left 35% of the page; the table and a summary box on the right."""
    rows = ledger(3, 10, 0, date(2026, 5, 4))
    previous = 4_500_000
    debit, credit, _, _ = totals(rows)
    due = previous + debit - credit
    pdf = Pdf()
    pdf.new_page()
    for i, line in enumerate(["FAKE PERSON NAME", "12 Sample Street", "Test City 000000",
                              "Statement Date 28/05/2026", "Payment Due Date 15/06/2026",
                              "Card No: 5200XXXXXXXX3333", "Reward Points 1,234",
                              "Offers and promotions 01/06/2026 12,000.00"]):
        pdf.text(30, 120 + i * 14, line)
    for i in range(10):
        pdf.text(30, 300 + i * 18, f"{rows[i].date.strftime('%d/%m/%Y')} 9,999.00 junk")
    pdf.text(215, 60, ISSUER, 12, bold=True)
    pdf.text(215, 76, "CREDIT CARD STATEMENT")
    pdf.text(215, 90, "Statement Period : May 01, 2026 to May 31, 2026")
    pdf.rect(215, 105, 560, 160)
    labels = ["Previous Balance", "Purchases", "Payments", "Total Amount Due"]
    values = [previous, debit, credit, due]
    for i, (label, value) in enumerate(zip(labels, values, strict=True)):
        x = 220 + i * 86
        pdf.text(x, 122, label, 7)
        pdf.text(x, 140, indian(value))
    pdf.text(215, 200, "Date", bold=True)
    pdf.text(270, 200, "Transaction Details", bold=True)
    pdf.text(545, 200, "Amount (Rs.)", bold=True, right=True)
    for i, row in enumerate(rows):
        y = 218 + i * 18
        pdf.text(215, y, row.date.strftime("%d/%m/%Y"))
        pdf.text(270, y, row.description[:42])
        pdf.text(540, y, indian(row.debit or row.credit), right=True)
        if row.credit:
            pdf.text(544, y, "CR")
        row.description = row.description[:42]
        row.balance = None
    pdf.save(out / "side_panel.pdf")
    stmt = statement(ISSUER, "credit_card", "5200XXXXXXXX3333",
                     (date(2026, 5, 1), date(2026, 5, 31)), rows, opening_paise=previous,
                     closing_paise=due, total_debit_paise=debit, total_credit_paise=credit)
    return Built("side_panel.pdf", expected_statements([stmt]),
                 {"format": "pdf", "pages": 1, "words": pdf.words, "panel_right_edge": 208})


def encrypted_pdf(out: Path) -> Built:
    """(i) The ruled layout, AES-encrypted. Password 'Synth1234' (the upper/lower variants fail)."""
    import pymupdf

    options = {"encryption": pymupdf.PDF_ENCRYPT_AES_256, "user_pw": "Synth1234",
               "owner_pw": "OwnerSynth99"}
    built = ruled_simple(out, "encrypted.pdf", seed=9, encryption=options)
    built.facts |= {"encrypted": True, "password": "Synth1234", "saved_password": "synth1234"}
    return built

