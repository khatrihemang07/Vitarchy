"""Structured stderr logs: stage names, counts and durations only (never content or secrets)."""

from __future__ import annotations

import json
import os
import sys
import time
from collections.abc import Iterator
from contextlib import contextmanager

REQUEST_ID_ENV = "STATEMENT_READER_REQUEST_ID"


def emit(event: str, **fields: int | str | bool | None) -> None:
    """One JSON line. Callers pass identifiers and counts, never entered values."""
    line = {"event": event, **fields}
    request_id = os.environ.get(REQUEST_ID_ENV)
    if request_id:
        line["request_id"] = request_id
    print(json.dumps(line), file=sys.stderr, flush=True)


@contextmanager
def stage(name: str) -> Iterator[dict]:
    """Time a pipeline stage; the body may add counts to the yielded dict."""
    counts: dict = {}
    started = time.perf_counter()
    try:
        yield counts
    finally:
        emit("stage", stage=name, duration_ms=round((time.perf_counter() - started) * 1000),
             **counts)
