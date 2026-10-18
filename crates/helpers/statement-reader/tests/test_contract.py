"""The dataclasses must serialise to exactly the shared contract fixtures (Rust tests read them too)."""

import json
from pathlib import Path

import pytest

from statement_reader.contract import (
    Account,
    Check,
    Diagnostics,
    PaymentRecord,
    Period,
    Request,
    Result,
    Statement,
    Summary,
    Transaction,
)

CONTRACT_DIR = Path(__file__).parent.parent / "fixtures" / "contract"


def load(name: str) -> dict:
    return json.loads((CONTRACT_DIR / f"{name}.json").read_text())


def read_statement() -> Result:
    transactions = [
        Transaction(
            date="2026-07-02",
            description="UPI/DR/512345678901/DEMO SHOP/EXB/pay./Indi",
            amount_paise=550000,
            direction="debit",
            balance_paise=8021778,
            reference="512345678901",
        ),
        Transaction(
            date="2026-07-03",
            description="OVERSEAS STORE LONDON GB",
            amount_paise=207500,
            direction="debit",
            foreign_amount="25.00",
            foreign_currency="USD",
            addon_card="4000XXXXXXXX2222",
            source_id="51234567907",
        ),
        Transaction(
            date="2026-07-04",
            description="PAYMENT RECEIVED",
            amount_paise=100000,
            direction="credit",
        ),
    ]
    statement = Statement(
        account=Account(issuer="Example Bank", kind="credit_card", mask="4000XXXXXXXX1111"),
        period=Period(start="2026-06-19", end="2026-07-18"),
        summary=Summary(
            opening_paise=1234500,
            closing_paise=99900,
            total_debit_paise=500000,
            total_credit_paise=100000,
            debit_count=2,
            credit_count=1,
        ),
        transactions=transactions,
        checks=[
            Check("balance_chain", "passed", {"rows": 3, "breaks": 0, "polarity": "normal"}),
            Check("summary_totals", "failed", {"debit_diff_paise": 100, "credit_diff_paise": 0}),
            Check("row_counts", "skipped"),
        ],
    )
    return Result(
        outcome="read",
        format="pdf",
        encrypted=True,
        unlocked_with=2,
        kind="statement",
        statements=[statement],
        diagnostics=Diagnostics(pages=4, rows=3),
    )


def payment_history() -> Result:
    records = [
        PaymentRecord(
            occurred_at="2026-10-18T11:20:00",
            description="Paid to SOME MERCHANT",
            amount_paise=25000,
            direction="debit",
            source_id="T2610181120000000000001",
            utr="512345678901",
            instrument="XXXXXX4321",
            instrument_digits="4321",
        ),
        PaymentRecord(
            occurred_at="2026-09-19T00:05:00",
            description="Received from SOME PERSON",
            amount_paise=120050,
            direction="credit",
        ),
    ]
    return Result(
        outcome="read",
        format="csv",
        kind="payment_history",
        payment_records=records,
        diagnostics=Diagnostics(rows=2),
    )


def locked() -> Result:
    return Result(
        outcome="locked",
        format="pdf",
        encrypted=True,
        diagnostics=Diagnostics(stage="open", code="no_password_worked"),
    )


def no_table() -> Result:
    return Result(
        outcome="no_table",
        format="xlsx",
        diagnostics=Diagnostics(stage="locate", code="no_header_or_columns", rows=40),
    )


SAMPLES = {
    "read_statement": read_statement,
    "payment_history": payment_history,
    "locked": locked,
    "no_table": no_table,
}


@pytest.mark.parametrize("name", SAMPLES)
def test_serialising_the_dataclasses_reproduces_the_shared_fixture(name):
    assert SAMPLES[name]().to_dict() == load(name)


@pytest.mark.parametrize("name", SAMPLES)
def test_to_json_is_valid_json_with_the_same_content(name):
    assert json.loads(SAMPLES[name]().to_json()) == load(name)


def test_request_reads_the_stdin_shape():
    request = Request.from_json(
        '{"version": 1, "file_path": "/x", "passwords": ["a", "b"], "decrypted_out": "/y"}'
    )
    assert (request.file_path, request.passwords, request.decrypted_out) == ("/x", ["a", "b"], "/y")


def test_request_defaults_missing_optional_fields():
    request = Request.from_json('{"version": 1, "file_path": "/x"}')
    assert request.passwords == [] and request.decrypted_out is None


def test_request_rejects_an_unknown_version():
    with pytest.raises(ValueError, match="version"):
        Request.from_json('{"version": 2, "file_path": "/x"}')
