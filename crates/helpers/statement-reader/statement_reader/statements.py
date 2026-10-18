"""Regions -> contract Statements: grouping by account, rows, facts, normalising and Checks."""

from __future__ import annotations

from dataclasses import replace
from datetime import date

from .checks.figures import balance_chain, row_counts, summary_totals
from .contract import Account, Period, Scope, Statement, Transaction
from .locate.regions import Region
from .metadata.facts import Facts, read_facts
from .metadata.periods import fill_period
from .metadata.surface import Surface
from .normalize.rows import Built, build_transactions
from .segment.card_sections import CardSection, split_by_card
from .segment.rows import RawRow, segment_region
from .vocabulary import Vocabulary


def group_regions(regions: list[Region]) -> list[list[Region]]:
    """One group per account: a region whose heading names another account starts a new group."""
    groups: list[list[Region]] = []
    mask: str | None = None
    for region in regions:
        heading = region.heading
        if not groups or (heading and mask and heading.mask != mask):
            groups.append([])
            mask = None
        groups[-1].append(region)
        mask = mask or (heading.mask if heading else None)
    return groups


def segment_group(group: list[Region], vocab: Vocabulary) -> list[RawRow]:
    rows: list[RawRow] = []
    card: str | None = None
    for region in group:
        segmented = segment_region(region.lines, region.layout, vocab, card)
        card = segmented.card
        rows.extend(segmented.rows)
    return rows


def iso(value: date | None) -> str | None:
    return value.isoformat() if value else None


def _statement(section: CardSection, built: Built, everything: list[Transaction], facts: Facts,
               period: tuple[date | None, date | None], scope: Scope | None) -> Statement:
    summary = replace(facts.summary)
    if summary.opening_paise is None:
        summary.opening_paise = built.opening_paise
    checks = [balance_chain(built.transactions, summary.opening_paise),
              summary_totals(everything, summary, facts.kind, facts.charges, scope),
              row_counts(everything, summary, scope)]
    return Statement(Account(facts.issuer, facts.kind, section.mask or facts.mask),
                     Period(iso(period[0]), iso(period[1])), summary, built.transactions, checks)


def make_statements(rows: list[RawRow], facts: Facts, surface: Surface, vocab: Vocabulary) -> list[Statement]:
    """One Statement per card section of `rows`, each normalised and Checked. A bill prints one
    summary for all its cards, so its totals and counts are Checked over every section's rows
    (Scope.BILL); the balance chain stays per section."""
    period = fill_period(facts.period, surface, vocab)
    sections = split_by_card(rows, facts.mask)
    scope = Scope.BILL if len(sections) > 1 else None
    built = [build_transactions(s.rows, vocab, facts.kind, period, facts.summary.opening_paise)
             for s in sections]
    everything = [t for b in built for t in b.transactions]
    return [_statement(s, b, everything, facts, period, scope) for s, b in zip(sections, built, strict=True)]


def statement_facts(surface: Surface, group_heading, vocab: Vocabulary) -> Facts:
    return read_facts(surface, vocab, group_heading.mask if group_heading else None,
                      group_heading.kind if group_heading else None)
