"""Every synthetic fixture reads back to exactly the Transactions planted in it."""

import json

import pytest

from statement_reader.cli import process
from statement_reader.contract import Request

from .conftest import FILES, MANIFEST

PASSWORDS = ["synth1234", "xlsx2026"]
NAMES = sorted(MANIFEST)


def read(name: str) -> dict:
    result = process(Request(file_path=str(FILES / name), passwords=PASSWORDS))
    return json.loads(result.to_json())


@pytest.mark.parametrize("name", NAMES)
def test_the_fixture_reads_to_the_planted_transactions(name):
    expected = json.loads((FILES / "expected" / f"{name.rsplit('.', 1)[0]}.json").read_text())
    got = read(name)
    assert got["outcome"] == expected["outcome"] and got["kind"] == expected["kind"], got["diagnostics"]
    assert len(got["statements"]) == len(expected.get("statements", []))
    for want, have in zip(expected.get("statements", []), got["statements"], strict=True):
        assert have["account"] == want["account"]
        assert have["period"] == want["period"]
        assert have["summary"] == want["summary"]
        assert have["transactions"] == want["transactions"]


def test_each_card_of_a_bill_is_its_own_statement_with_bill_scope_totals():
    got = read("multi_card_bill.pdf")
    first, second = got["statements"]
    assert [first["account"]["mask"], second["account"]["mask"]] == ["4000XXXXXXXX1111", "4000XXXXXXXX2222"]
    for statement in got["statements"]:
        checks = {c["name"]: c for c in statement["checks"]}
        assert checks["summary_totals"]["outcome"] == "passed"
        assert checks["summary_totals"]["detail"]["scope"] == "bill"
        assert "scope" not in checks["balance_chain"]["detail"]


def test_a_consolidated_statement_keeps_its_checks_per_account():
    for statement in read("consolidated.pdf")["statements"]:
        assert all("scope" not in c["detail"] for c in statement["checks"])
