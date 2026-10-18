"""The UPI RRN/UTR or NEFT/IMPS reference inside a description, found with the vocabulary's patterns."""

from __future__ import annotations

from ..vocabulary import Vocabulary


def find_reference(description: str, vocab: Vocabulary) -> str | None:
    """The first pattern (in vocabulary order) that matches decides; its group 1 is the reference."""
    for pattern in vocab.reference_patterns:
        match = pattern.search(description)
        if match:
            return match.group(1)
    return None
