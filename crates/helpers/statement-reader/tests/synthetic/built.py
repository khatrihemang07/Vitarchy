"""What a fixture builder returns: the file, the Transactions planted in it, and facts to test against."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import date

from .data import Row, transaction

SUMMARY_KEYS = (
    "opening_paise", "closing_paise", "total_debit_paise", "total_credit_paise",
    "debit_count", "credit_count",
)


@dataclass
class Built:
    filename: str
    expected: dict  # a contract-shaped Result the engine should reproduce
    facts: dict = field(default_factory=dict)  # format, pages, words / rows, cols, password


def statement(issuer: str | None, kind: str | None, mask: str | None,
              period: tuple[date | None, date | None], rows: list[Row],
              extras: dict[int, dict] | None = None, **summary) -> dict:
    """A planted Statement. `extras` maps a row index to per-row contract fields."""
    extras = extras or {}
    return {
        "account": {"issuer": issuer, "kind": kind, "mask": mask},
        "period": {"start": period[0] and period[0].isoformat(),
                   "end": period[1] and period[1].isoformat()},
        "summary": {key: summary.get(key) for key in SUMMARY_KEYS},
        "transactions": [transaction(r, **extras.get(i, {})) for i, r in enumerate(rows)],
    }


def expected_statements(statements: list[dict]) -> dict:
    return {"outcome": "read", "kind": "statement", "statements": statements}
