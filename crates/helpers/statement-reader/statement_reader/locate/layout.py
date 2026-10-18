"""Where a table's columns sit on the page: what locate hands to segment."""

from __future__ import annotations

from dataclasses import dataclass

MONEY_FIELDS = ("debit", "credit", "amount", "balance")
NUMERIC_FIELDS = (*MONEY_FIELDS, "foreign_amount")
DATE_FIELDS = ("date", "value_date")


@dataclass(frozen=True)
class Column:
    field: str
    x0: float
    x1: float

    @property
    def centre(self) -> float:
        return (self.x0 + self.x1) / 2


@dataclass
class Layout:
    """Columns left to right. A field may appear twice; the first one wins in lookups."""

    columns: list[Column]

    def __post_init__(self) -> None:
        self.columns = sorted(self.columns, key=lambda c: c.x0)

    def get(self, field: str) -> Column | None:
        return next((c for c in self.columns if c.field == field), None)

    @property
    def fields(self) -> set[str]:
        return {c.field for c in self.columns}

    @property
    def date_column(self) -> Column | None:
        return next((c for f in DATE_FIELDS if (c := self.get(f))), None)

    @property
    def x_min(self) -> float:
        return min(c.x0 for c in self.columns)

    @property
    def x_max(self) -> float:
        return max(c.x1 for c in self.columns)


def has_date_and_money(fields: set[str]) -> bool:
    return bool(fields & set(DATE_FIELDS)) and bool(fields & set(MONEY_FIELDS))
