"""The reading engine: a Document in, a contract Result out. Every layout is handled by the same
stages; what differs between issuers lives in vocabulary.toml, never here."""

from __future__ import annotations

from .contract import Result
from .document import Document
from .pdf_flow import read_pdf
from .sheet_flow import read_sheet


def read_document(doc: Document) -> Result:
    result = read_pdf(doc.pages) if doc.pages else read_sheet(doc.grid or [])
    return Result(**{**result.__dict__, "format": doc.format})

