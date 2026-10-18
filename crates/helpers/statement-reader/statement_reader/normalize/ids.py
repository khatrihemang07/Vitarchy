"""Row ids: tell an issuer's per-Transaction id from a plain row counter, whatever the header says."""

from __future__ import annotations

from ..segment.rows import RawRow

MIN_ID_LENGTH = 8  # an id this long (digits or letters) is no counter


def _values(rows: list[RawRow], field: str) -> list[str]:
    return [r.cells.get(field, "").strip() for r in rows]


def is_counter(values: list[str]) -> bool:
    """1, 2, 3 ... in order (or nothing at all)."""
    return all(v.isdigit() and int(v) == i for i, v in enumerate(values, start=1)) or not any(values)


def is_id_column(values: list[str]) -> bool:
    """Every row has a long value and no two rows share one."""
    return (bool(values) and all(len(v) >= MIN_ID_LENGTH for v in values)
            and len(set(values)) == len(values))


def resolve_ids(rows: list[RawRow]) -> None:
    """Moves long unique ids from `serial` into `source_id` and drops counters in `source_id`."""
    serial, source = _values(rows, "serial"), _values(rows, "source_id")
    if rows and "source_id" in rows[0].cells and is_counter(source):
        for row in rows:
            row.cells.pop("source_id", None)
    if rows and "serial" in rows[0].cells and is_id_column(serial):
        for row in rows:
            row.cells["source_id"] = row.cells.pop("serial")
