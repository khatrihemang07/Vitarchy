"""Dataclasses mirroring CONTRACT.md v1: the single source of the schema on the Python side."""

from __future__ import annotations

import json
from dataclasses import asdict, dataclass, field
from enum import Enum
from typing import Any

CONTRACT_VERSION = 1


@dataclass
class Request:
    """What the server writes to stdin."""

    file_path: str
    passwords: list[str] = field(default_factory=list)
    decrypted_out: str | None = None
    version: int = CONTRACT_VERSION

    @classmethod
    def from_json(cls, text: str) -> Request:
        data = json.loads(text)
        if data.get("version") != CONTRACT_VERSION:
            raise ValueError(f"unsupported request version: {data.get('version')!r}")
        return cls(
            file_path=data["file_path"],
            passwords=list(data.get("passwords") or []),
            decrypted_out=data.get("decrypted_out"),
        )


@dataclass
class Account:
    issuer: str | None = None
    kind: str | None = None  # savings | credit_card | loan
    mask: str | None = None


@dataclass
class Period:
    start: str | None = None
    end: str | None = None


@dataclass
class Summary:
    opening_paise: int | None = None
    closing_paise: int | None = None
    total_debit_paise: int | None = None
    total_credit_paise: int | None = None
    debit_count: int | None = None
    credit_count: int | None = None


@dataclass
class Transaction:
    date: str
    description: str
    amount_paise: int
    direction: str  # debit | credit
    balance_paise: int | None = None
    foreign_amount: str | None = None
    foreign_currency: str | None = None
    reference: str | None = None
    source_id: str | None = None


class Scope(Enum):
    """Check detail: what the printed figures cover beyond one Statement's rows."""

    BILL = "bill"  # every card on the bill


@dataclass
class Check:
    name: str  # balance_chain | summary_totals | row_counts
    outcome: str  # passed | failed | skipped
    detail: dict[str, Any] = field(default_factory=dict)


@dataclass
class Statement:
    account: Account = field(default_factory=Account)
    period: Period = field(default_factory=Period)
    summary: Summary = field(default_factory=Summary)
    transactions: list[Transaction] = field(default_factory=list)
    checks: list[Check] = field(default_factory=list)


@dataclass
class PaymentRecord:
    occurred_at: str
    description: str
    amount_paise: int
    direction: str
    source_id: str | None = None
    utr: str | None = None
    instrument: str | None = None
    instrument_issuer: str | None = None  # the issuer the vocabulary recognises in `instrument`
    instrument_digits: str | None = None  # the trailing visible digits of `instrument`


@dataclass
class Diagnostics:
    stage: str | None = None  # open | read | locate | segment | normalize
    code: str | None = None
    pages: int | None = None
    rows: int | None = None


@dataclass
class Result:
    """What the helper writes to stdout."""

    outcome: str  # read | locked | unsupported | no_table
    format: str | None = None  # pdf | csv | xls | xlsx
    encrypted: bool = False
    unlocked_with: int | None = None
    kind: str | None = None  # statement | payment_history
    statements: list[Statement] = field(default_factory=list)
    payment_records: list[PaymentRecord] = field(default_factory=list)
    diagnostics: Diagnostics = field(default_factory=Diagnostics)
    version: int = CONTRACT_VERSION

    def to_dict(self) -> dict[str, Any]:
        # Field order is the contract's order: version first, diagnostics last.
        data = asdict(self)
        order = ["version", *(k for k in data if k != "version")]
        return {k: data[k] for k in order}

    def to_json(self) -> str:
        return json.dumps(self.to_dict(), ensure_ascii=False)
