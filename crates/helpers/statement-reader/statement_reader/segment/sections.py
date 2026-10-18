"""Section markers inside and between tables: card sections and account headings."""

from __future__ import annotations

from dataclasses import dataclass

from ..read.lines import Line
from ..vocabulary import Vocabulary


def card_marker(line: Line, vocab: Vocabulary) -> str | None:
    """The masked card number when it is alone on the line ("4000XXXXXXXX2222"), else None."""
    text = line.text.strip()
    return text if vocab.masked_number.fullmatch(text) else None


def is_cardholder_divider(line: Line, vocab: Vocabulary) -> bool:
    return bool(vocab.cardholder_divider.search(line.text.strip()))


@dataclass(frozen=True)
class Heading:
    """An account heading such as "DL/TL ACCOUNT xxxxxxx1470" or "Savings Account XXXXXXXX8765"."""

    mask: str
    kind: str | None
    y: float


HEADING_LINES = 25.0  # the account label and its number may sit on two lines this far apart


def _heading_in(text: str, y: float, vocab: Vocabulary) -> Heading | None:
    mask = vocab.masked_account.search(text)
    kinds = [kind for pattern, kind in vocab.account_label_kinds if pattern.search(text)]
    return Heading(mask.group(0), kinds[0], y) if mask and kinds else None


def find_heading(lines: list[Line], vocab: Vocabulary) -> Heading | None:
    """The last account heading among `lines` (the one closest above a table), or None."""
    for index in range(len(lines) - 1, -1, -1):
        line = lines[index]
        if found := _heading_in(line.text, line.y, vocab):
            return found
        if index > 0 and line.y - lines[index - 1].y <= HEADING_LINES:
            joined = f"{lines[index - 1].text} {line.text}"
            if found := _heading_in(joined, line.y, vocab):
                return found
    return None
