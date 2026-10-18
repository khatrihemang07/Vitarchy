"""Loads vocabulary.toml: the data that tells the engine what words, formats and markers mean."""

from __future__ import annotations

import re
import tomllib
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path

DEFAULT_PATH = Path(__file__).with_name("vocabulary.toml")
PREFIX_MIN_LENGTH = 5  # shorter synonyms (such as "dr") only match a whole header cell


def normalise(text: str) -> str:
    """Lower case, letters and digits only: how header cells and synonyms are compared."""
    return re.sub(r"[^a-z0-9]", "", text.lower())


@dataclass(frozen=True)
class IssuerMarker:
    name: str
    names: tuple[str, ...]
    ifsc_prefix: tuple[str, ...]
    gstin_pan: tuple[str, ...]
    kind: str


class Vocabulary:
    def __init__(self, data: dict) -> None:
        self.data = data
        self.fields: dict[str, tuple[str, ...]] = {k: tuple(v) for k, v in data["fields"].items()}
        self._lookup = {normalise(s): f for f, words in self.fields.items() for s in words}
        self.date_formats: tuple[str, ...] = tuple(data["dates"]["formats"])
        self.time_formats: tuple[str, ...] = tuple(data["dates"]["time_formats"])
        self.month_aliases: dict[str, str] = dict(data["dates"]["month_aliases"])
        self.directions: dict[str, tuple[str, ...]] = {
            k: tuple(v) for k, v in data["directions"].items()}
        self.currency_marks: tuple[str, ...] = tuple(data["amounts"]["currency_marks"])
        self.zero_marks: tuple[str, ...] = tuple(data["amounts"]["zero_marks"])
        self.rows: dict[str, tuple[str, ...]] = {k: tuple(v) for k, v in data["rows"].items()}
        self.stop_startswith: tuple[str, ...] = tuple(data["stops"]["startswith"])
        self.stop_contains: tuple[str, ...] = tuple(data["stops"]["contains"])
        sections = data["sections"]
        self.masked_number = re.compile(sections["masked_number"])
        self.masked_account = re.compile(sections["masked_account"])
        self.cardholder_divider = re.compile(sections["cardholder_divider"])
        self.account_label_kinds = tuple(
            (re.compile(e["pattern"]), e["kind"]) for e in sections["account_label_kinds"])
        self.kind_phrases: dict[str, tuple[str, ...]] = {
            k: tuple(v) for k, v in data["kinds"].items()}
        self.mask_token = re.compile(sections["mask_token"])
        self.noise_startswith: tuple[str, ...] = self.rows["noise_startswith"]
        self.facts: dict[str, tuple[str, ...]] = {k: tuple(v) for k, v in data["facts"].items()}
        self.period_labels: tuple[str, ...] = tuple(data["dates"]["period_labels"])
        self.period_separators: tuple[str, ...] = tuple(data["dates"]["period_separators"])
        self.payment_history: dict = data["payment_history"]
        self.reference_patterns = tuple(re.compile(p) for p in data["references"]["patterns"])
        self.issuers = tuple(
            IssuerMarker(i["name"], tuple(i["names"]), tuple(i["ifsc_prefix"]), tuple(i["gstin_pan"]),
                         i["kind"]) for i in data["issuers"])
        self.instrument_issuers: tuple[tuple[str, tuple[str, ...]], ...] = tuple(
            (i["name"], tuple(i["names"])) for i in data["instrument_issuers"])

    def field_of(self, header: str, allow_prefix: bool = True) -> str | None:
        """The canonical field a header cell names, or None. Whole-cell matches win over prefixes."""
        cell = normalise(header)
        if not cell:
            return None
        if cell in self._lookup:
            return self._lookup[cell]
        if not allow_prefix:
            return None
        if "credit" in cell and "debit" in cell:
            return None  # e.g. "credit/debit instrument" must not read as a credit column
        for synonym in sorted(self._lookup, key=len, reverse=True):
            if len(synonym) >= PREFIX_MIN_LENGTH and cell.startswith(synonym):
                return self._lookup[synonym]
        return None

    def is_stop_line(self, line: str) -> bool:
        text = line.strip().lower()
        return text.startswith(self.stop_startswith) or any(t in text for t in self.stop_contains)


@lru_cache(maxsize=1)
def load_vocabulary(path: Path = DEFAULT_PATH) -> Vocabulary:
    with open(path, "rb") as handle:
        return Vocabulary(tomllib.load(handle))
