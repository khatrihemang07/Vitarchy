"""Fake ledgers: deterministic rows with a running balance, plus the money formats the layouts print."""

from __future__ import annotations

import random
import re
from dataclasses import dataclass
from datetime import date, timedelta

MERCHANTS = [
    "ACME GROCERS", "DEMO CAFE", "SAMPLE PHARMACY", "TEST BOOKS", "FAKE FUELS",
    "NOVA MART", "ZETA TRAVELS", "LUMEN ELECTRIC", "PIXEL STORE", "ORCHID SWEETS",
]


@dataclass
class Row:
    date: date
    description: str
    debit: int = 0  # paise
    credit: int = 0
    balance: int | None = None


def indian(paise: int) -> str:
    """12345678 -> '1,23,456.78' (lakh grouping); a negative amount keeps its sign."""
    if paise < 0:
        return "-" + indian(-paise)
    whole, frac = divmod(paise, 100)
    digits = str(whole)
    head, tail = digits[:-3], digits[-3:]
    groups = []
    while head:
        groups.insert(0, head[-2:])
        head = head[:-2]
    return ",".join([*groups, tail]) + f".{frac:02d}"


def plain(paise: int) -> str:
    if paise < 0:
        return "-" + plain(-paise)
    return f"{paise // 100}.{paise % 100:02d}"


def ledger(seed: int, count: int, opening: int, start: date, inverted: bool = False,
           big_at: int | None = None) -> list[Row]:
    """Rows with a running balance; `inverted` is a loan (credit reduces what is owed)."""
    rng = random.Random(seed)
    rows, balance, day = [], opening, start
    for i in range(count):
        day += timedelta(days=rng.choice([0, 1, 1, 2]))
        name = rng.choice(MERCHANTS)
        ref = "".join(rng.choice("0123456789") for _ in range(12))
        amount = rng.randrange(5_000, 999_999)
        if i == big_at:
            amount = 12_345_678
        is_credit = rng.random() < 0.25
        row = Row(day, f"UPI/{ref}/{name}" if rng.random() < 0.5 else name)
        if is_credit:
            row.credit = amount
        else:
            row.debit = amount
        sign = (row.credit - row.debit) * (-1 if inverted else 1)
        balance += sign
        row.balance = balance
        rows.append(row)
    return rows


def totals(rows: list[Row]) -> tuple[int, int, int, int]:
    """(debit paise, credit paise, debit count, credit count)."""
    return (
        sum(r.debit for r in rows), sum(r.credit for r in rows),
        sum(1 for r in rows if r.debit), sum(1 for r in rows if r.credit),
    )


def planted_reference(description: str) -> str | None:
    """The 12-digit id the generator put after 'UPI/' (what a reader should derive)."""
    match = re.search(r"UPI/(\d{12})/", description)
    return match.group(1) if match else None


def transaction(row: Row, **extra) -> dict:
    """The contract Transaction the engine should produce for a planted row."""
    return {
        "date": row.date.isoformat(),
        "description": row.description,
        "amount_paise": row.debit or row.credit,
        "direction": "debit" if row.debit else "credit",
        "balance_paise": row.balance,
        "foreign_amount": None, "foreign_currency": None,
        "reference": planted_reference(row.description), "source_id": None, "time": None,
        **extra,
    }
