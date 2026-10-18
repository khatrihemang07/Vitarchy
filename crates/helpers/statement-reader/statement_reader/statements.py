"""Regions -> contract Statements: grouping by account, rows, facts, normalising and Checks."""

from __future__ import annotations

from datetime import date

from .checks.figures import balance_chain, row_counts, summary_totals
from .contract import Account, Period, Statement
from .locate.regions import Region
from .metadata.facts import Facts, read_facts
from .metadata.periods import fill_period
from .metadata.surface import Surface
from .normalize.rows import Built, build_transactions
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


def make_statement(rows: list[RawRow], facts: Facts, surface: Surface, vocab: Vocabulary) -> Statement:
    """Normalise `rows`, then fill the summary and run the Checks."""
    period = fill_period(facts.period, surface, vocab)
    built: Built = build_transactions(rows, vocab, facts.kind, period,
                                      facts.summary.opening_paise, facts.mask)
    summary = facts.summary
    if summary.opening_paise is None:
        summary.opening_paise = built.opening_paise
    transactions = built.transactions
    checks = [balance_chain(transactions, summary.opening_paise),
              summary_totals(transactions, summary, facts.kind, facts.charges),
              row_counts(transactions, summary)]
    return Statement(Account(facts.issuer, facts.kind, facts.mask), Period(iso(period[0]), iso(period[1])),
                     summary, transactions, checks)


def statement_facts(surface: Surface, group_heading, vocab: Vocabulary) -> Facts:
    return read_facts(surface, vocab, group_heading.mask if group_heading else None,
                      group_heading.kind if group_heading else None)
