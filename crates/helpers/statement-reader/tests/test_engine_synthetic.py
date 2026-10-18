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
    assert got["payment_records"] == expected.get("payment_records", [])
    assert len(got["statements"]) == len(expected.get("statements", []))
    for want, have in zip(expected.get("statements", []), got["statements"], strict=True):
        assert have["account"] == want["account"]
        assert have["period"] == want["period"]
        assert have["summary"] == want["summary"]
        assert have["transactions"] == want["transactions"]
