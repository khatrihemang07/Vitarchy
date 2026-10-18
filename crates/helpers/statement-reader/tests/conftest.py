"""Shared fixtures: the committed synthetic files and their manifest."""

import json
import shutil
from pathlib import Path

import pytest

FILES = Path(__file__).parent / "synthetic" / "files"
MANIFEST = json.loads((FILES / "manifest.json").read_text())
PDFS = sorted(n for n, f in MANIFEST.items() if f["format"] == "pdf" and not f.get("encrypted"))


@pytest.fixture
def synthetic() -> Path:
    return FILES


@pytest.fixture
def copy_of(tmp_path):
    """Copy a synthetic file under another name, e.g. to test that the extension is ignored."""
    def make(name: str, as_name: str) -> str:
        target = tmp_path / as_name
        shutil.copy(FILES / name, target)
        return str(target)
    return make
