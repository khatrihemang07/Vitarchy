"""The facts a Statement prints about itself: period, balances, totals, counts, account, issuer."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import date

from ..contract import Summary
from ..vocabulary import Vocabulary
from .issuer import detect_issuer, detect_kind
from .labels import find_value
from .parsers import count_value, first_mask, mask_value, money_value, period_value, total_value
from .periods import unlabelled_period
from .surface import Surface

# summary field -> (vocabulary facts key, value parser)
SUMMARY_FIELDS = {
    "opening_paise": ("opening", money_value), "closing_paise": ("closing", money_value),
    "total_debit_paise": ("total_debit", total_value), "total_credit_paise": ("total_credit", total_value),
    "debit_count": ("debit_count", count_value), "credit_count": ("credit_count", count_value),
}


@dataclass
class Facts:
    summary: Summary = field(default_factory=Summary)
    charges: int = 0  # fees and interest printed in the summary, outside the rows
    period: tuple[date | None, date | None] = (None, None)
    mask: str | None = None
    issuer: str | None = None
    kind: str | None = None


def _mask_in_sentence(surface: Surface, vocab: Vocabulary) -> str | None:
    """A number after a mask label inside running text ("Saving Account no. 123456789012 in INR"),
    searched only up to the next label, and only when it follows the label directly, so a phone
    number later in the sentence ("card number> to 9215...") is never taken."""
    labels = vocab.facts["mask"]

    def parse(text: str) -> str | None:
        low = text.lower()
        cut = min((i for label in labels if (i := low.find(label)) >= 0), default=len(text))
        after = text[:cut].lstrip(" :.#-")
        found = mask_value(after, vocab)
        return found if found and after.startswith(found.split()[0]) else None

    return find_value(surface, labels, parse, contains=True)


def _header_mask(surface: Surface, vocab: Vocabulary) -> str | None:
    """A masked number printed inside other text. A number alone on its line is an add-on card's
    section marker, which only serves as the mask when nothing else names the Account."""
    prose = " ".join(c.text for c in surface.cells if not vocab.masked_number.fullmatch(c.text.strip()))
    return first_mask(prose, vocab)


def read_facts(surface: Surface, vocab: Vocabulary, heading_mask: str | None = None,
               heading_kind: str | None = None) -> Facts:
    """Reads the labelled figures in `surface`. The account heading, when there is one, names the account."""
    facts = Facts()
    for name, (key, parser) in SUMMARY_FIELDS.items():
        setattr(facts.summary, name,
                find_value(surface, vocab.facts[key], lambda t, p=parser: p(t, vocab)))
    facts.charges = find_value(surface, vocab.facts["charges"], lambda t: total_value(t, vocab)) or 0
    period = find_value(surface, vocab.period_labels, lambda t: period_value(t, vocab), contains=True)
    facts.period = period or unlabelled_period(surface, vocab) or (None, None)
    text = " ".join(c.text for c in surface.cells)
    facts.mask = (heading_mask or find_value(surface, vocab.facts["mask"], lambda t: mask_value(t, vocab))
                  or _mask_in_sentence(surface, vocab) or _header_mask(surface, vocab)
                  or first_mask(text, vocab))
    issuer = detect_issuer(text, vocab)
    facts.issuer = issuer.name if issuer else None
    facts.kind = heading_kind or detect_kind(text, vocab, issuer)
    return facts
