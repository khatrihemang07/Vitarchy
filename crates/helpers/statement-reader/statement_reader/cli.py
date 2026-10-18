"""statement-reader: a Request on stdin, a Result (contract v1) on stdout. Exit 0 unless it crashes."""

from __future__ import annotations

import dataclasses
import sys

from . import engine
from .contract import Diagnostics, Request, Result
from .document import Document
from .errors import OpenError, ReadError
from .log import emit, stage
from .open.sniff import Sniffed, sniff
from .open.unlock import Unlocked, unlock
from .read.pdf_words import open_pdf
from .read.sheet import read_grid


def stopped(outcome: str, stage_name: str, code: str, **fields) -> Result:
    return Result(outcome=outcome, diagnostics=Diagnostics(stage=stage_name, code=code), **fields)


def open_document(unlocked: Unlocked, sniffed: Sniffed) -> Document:
    with stage("read") as counts:
        if sniffed.format == "pdf":
            reader = open_pdf(unlocked.source)
            counts["pages"] = reader.page_count
            return Document("pdf", pages=reader)
        grid = read_grid(unlocked.source, sniffed.format or "")
        counts["rows"] = len(grid)
        counts["cols"] = len(grid[0]) if grid else 0
        return Document(sniffed.format or "", grid=grid)


def process(request: Request) -> Result:
    with stage("sniff") as counts:
        sniffed = sniff(request.file_path)
        counts["format"] = sniffed.format
    if sniffed.format is None:
        return stopped("unsupported", "open", "unsupported_format")
    try:
        with stage("unlock") as counts:
            unlocked = unlock(request.file_path, sniffed, request.passwords, request.decrypted_out)
            counts.update(encrypted=unlocked.encrypted, unlocked_with=unlocked.unlocked_with)
        if unlocked.locked:
            return stopped("locked", "open", "no_password_worked", format=sniffed.format,
                           encrypted=True)
        document = open_document(unlocked, sniffed)
    except OpenError as error:
        return stopped("unsupported", "open", error.code)
    except ReadError as error:
        return stopped("no_table", "read", error.code, format=sniffed.format)
    with stage("engine") as counts:
        result = engine.read_document(document)
        counts.update(outcome=result.outcome, statements=len(result.statements))
    if document.pages:
        document.pages.close()
    return dataclasses.replace(result, format=document.format, encrypted=unlocked.encrypted,
                               unlocked_with=unlocked.unlocked_with)


def main() -> int:
    try:
        request = Request.from_json(sys.stdin.read())
    except (ValueError, KeyError) as error:
        emit("bad_request", error=type(error).__name__)
        return 2
    result = process(request)
    emit("done", outcome=result.outcome)
    sys.stdout.write(result.to_json() + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
