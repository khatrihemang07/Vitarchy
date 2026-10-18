"""Who issued a Statement and what kind of Account it is, from the vocabulary's markers."""

from __future__ import annotations

import re

from ..vocabulary import IssuerMarker, Vocabulary

IFSC = re.compile(r"\b([A-Z]{4})0[A-Z0-9]{6}\b")
GSTIN_PAN = re.compile(r"\b\d{2}([A-Z]{5}\d{4}[A-Z])[0-9A-Z]Z[0-9A-Z]\b")


def _first_position(marker: IssuerMarker, text: str, lowered: str) -> int | None:
    found = [lowered.find(name) for name in marker.names if name in lowered]
    for pattern, wanted in ((IFSC, marker.ifsc_prefix), (GSTIN_PAN, marker.gstin_pan)):
        found += [m.start() for m in pattern.finditer(text) if m.group(1) in wanted]
    return min(found) if found else None


def detect_issuer(text: str, vocab: Vocabulary) -> IssuerMarker | None:
    """The issuer whose marker (name, IFSC prefix or GSTIN PAN) appears earliest in the text."""
    lowered = text.lower()
    hits = [(position, marker) for marker in vocab.issuers
            if (position := _first_position(marker, text, lowered)) is not None]
    return min(hits, key=lambda hit: hit[0])[1] if hits else None


def detect_kind(text: str, vocab: Vocabulary, issuer: IssuerMarker | None) -> str | None:
    """The Account kind whose vocabulary phrase appears earliest; else the issuer's usual kind."""
    lowered = text.lower()
    hits = [(lowered.find(phrase), kind) for kind, phrases in vocab.kind_phrases.items()
            for phrase in phrases if phrase in lowered]
    if hits:
        return min(hits)[1]
    return issuer.kind if issuer else None
