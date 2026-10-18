"""The committed synthetic files must match their generator, and their planted data must be sound."""

import json
from pathlib import Path

import pytest

from .conftest import FILES
from .synthetic.make_fixtures import BUILDERS


@pytest.fixture(scope="module")
def regenerated(tmp_path_factory):
    out = tmp_path_factory.mktemp("regen")
    return {build.__name__: build(out) for build in BUILDERS}


@pytest.mark.parametrize("build", BUILDERS, ids=lambda b: b.__name__)
def test_committed_expected_and_manifest_match_the_generator(build, regenerated):
    built = regenerated[build.__name__]
    stem = Path(built.filename).stem
    assert json.loads((FILES / "expected" / f"{stem}.json").read_text()) == built.expected
    assert json.loads((FILES / "manifest.json").read_text())[built.filename] == built.facts


def statements():
    for path in sorted((FILES / "expected").glob("*.json")):
        expected = json.loads(path.read_text())
        for stmt in expected.get("statements", []):
            yield path.stem, stmt


@pytest.mark.parametrize("name,stmt", [(n, s) for n, s in statements()])
def test_planted_balances_chain(name, stmt):
    rows = [t for t in stmt["transactions"] if t["balance_paise"] is not None]
    if not rows:
        return
    sign = -1 if stmt["account"]["kind"] == "loan" else 1
    previous = stmt["summary"]["opening_paise"]
    for row in rows:
        delta = row["amount_paise"] * (1 if row["direction"] == "credit" else -1) * sign
        if previous is not None:
            assert previous + delta == row["balance_paise"], name
        previous = row["balance_paise"]


def test_every_trait_has_a_fixture_with_expected_output():
    expected = {p.stem for p in (FILES / "expected").glob("*.json")}
    assert len(expected) == 15 and len(BUILDERS) == 15
