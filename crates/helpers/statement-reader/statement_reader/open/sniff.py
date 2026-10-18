"""Detects the file format from its bytes. The extension is never consulted."""

from __future__ import annotations

import zipfile
from dataclasses import dataclass

import olefile

OLE_MAGIC = bytes.fromhex("d0cf11e0a1b11ae1")
HEAD = 4096
# Control bytes that never appear in text (tab, newline, form feed and carriage return may).
BINARY_BYTES = bytes([*range(0, 9), 11, *range(14, 32)])


@dataclass(frozen=True)
class Sniffed:
    format: str | None  # pdf | csv | xls | xlsx | None when unsupported
    encrypted_ooxml: bool = False  # an OLE container holding an EncryptedPackage stream


def sniff(path: str) -> Sniffed:
    with open(path, "rb") as handle:
        head = handle.read(HEAD)
    if b"%PDF" in head[:1024]:
        return Sniffed("pdf")
    if head.startswith(OLE_MAGIC):
        return _sniff_ole(path)
    if head.startswith(b"PK"):
        return Sniffed("xlsx") if _is_workbook_zip(path) else Sniffed(None)
    return Sniffed("csv") if _looks_like_text(head) else Sniffed(None)


def _sniff_ole(path: str) -> Sniffed:
    try:
        with olefile.OleFileIO(path) as ole:
            if ole.exists("EncryptedPackage"):
                return Sniffed("xlsx", encrypted_ooxml=True)
            if ole.exists("Workbook") or ole.exists("Book"):
                return Sniffed("xls")
    except OSError:
        pass
    return Sniffed(None)


def _is_workbook_zip(path: str) -> bool:
    try:
        with zipfile.ZipFile(path) as archive:
            return "xl/workbook.xml" in archive.namelist()
    except zipfile.BadZipFile:
        return False


def _looks_like_text(head: bytes) -> bool:
    return bool(head.strip()) and not any(byte in BINARY_BYTES for byte in head)
