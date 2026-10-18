"""Joins the printed lines of one description back into the text the Statement printed."""

from __future__ import annotations

import re

SEPARATORS = "/@._-*:+=%"  # characters inside ids, urls and UPI handles
MIN_SPLIT_TOKEN = 8  # a token this long, ending a line, may have been cut by the wrap


OPEN_ENDINGS = "-_@."  # a long id ending like this was cut: nothing follows it after a space
LONG_ID = 20  # a token this long that ends a line is an id the wrap may have cut inside a hex run
HEX_FRAGMENT = re.compile(r"^[0-9a-f]{1,8}$")


def _id_like(token: str) -> bool:
    return any(c.isdigit() or c in SEPARATORS for c in token)


def continues_token(previous: str, following: str) -> bool:
    """True when `following` looks like the rest of the token that `previous` ends with.

    A wrap cuts text by width, not at spaces, so an id ("IBL458f1fb4ce" + "ae4f7fa2") arrives in
    two pieces and a space would corrupt it. Text alone cannot tell that from a wrap between two
    words, so we only glue when both sides look like part of an id: the last token before the break
    is long and holds a digit or an id separator, and the first token after it does too. Plain words
    ("NAIR", "IN") always get a space.
    """
    tail, head = previous.split()[-1], following.split()[0]
    if len(tail) < MIN_SPLIT_TOKEN or not _id_like(tail):
        return False
    if tail[-1] in OPEN_ENDINGS:  # "…/DELE-" + "Asha": nothing follows a joiner after a space
        return True
    return _id_like(head) or (len(tail) >= LONG_ID and HEX_FRAGMENT.match(head) is not None)


def join_description(lines: list[str], full: list[bool] | None = None) -> str:
    """Lines joined with one space, or with nothing where the wrap split a token.

    `full[i]` says line i reaches the wrap width. A token is only cut where the line is full, so a
    short line is never glued to the next one ("...Refund/HDFC" + "BANK/1037..." keeps its space).
    Without `full` every line counts as full.
    """
    text, was_full = "", True
    for index, raw in enumerate(lines):
        line = " ".join(raw.split())
        if not line:
            continue
        glue = text and (was_full or text[-1] in OPEN_ENDINGS) and continues_token(text, line)
        text = text + line if glue else f"{text} {line}".strip()
        was_full = True if full is None else full[index]
    return text


def cell_text(value: str) -> str:
    """A spreadsheet cell's text. A line break followed by one space is the issuer's hard wrap
    through a word or id, so the pieces are glued; any other line break is a space."""
    lines = value.split("\n")
    text = " ".join(lines[0].split())
    for raw in lines[1:]:
        piece = " ".join(raw.split())
        if not piece:
            continue
        cut_word = raw.startswith(" ") and not raw.startswith("  ")
        text = f"{text}{piece}" if cut_word else f"{text} {piece}".strip()
    return text
