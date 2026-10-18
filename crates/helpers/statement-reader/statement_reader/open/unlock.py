"""Opens locked files with the saved Statement passwords and their case variants."""

from __future__ import annotations

import io
import zipfile
from dataclasses import dataclass

import msoffcrypto
import pymupdf
from msoffcrypto.exceptions import InvalidKeyError

from ..errors import OpenError
from ..source import Source
from .sniff import Sniffed


@dataclass(frozen=True)
class Unlocked:
    source: Source  # the original path, or the decrypted bytes
    encrypted: bool
    unlocked_with: int | None  # index into the passwords list
    locked: bool = False  # encrypted and no password opened it


def password_attempts(passwords: list[str]) -> list[tuple[int, str]]:
    """(index, candidate) pairs: each password as-is, upper, lower and capitalised, once each."""
    seen: set[str] = set()
    attempts: list[tuple[int, str]] = []
    for index, password in enumerate(passwords):
        for candidate in (password, password.upper(), password.lower(), password.capitalize()):
            if candidate and candidate not in seen:
                seen.add(candidate)
                attempts.append((index, candidate))
    return attempts


def unlock(path: str, sniffed: Sniffed, passwords: list[str],
           decrypted_out: str | None) -> Unlocked:
    """Unlock a PDF or encrypted OOXML. A decrypted copy is written only if the file was encrypted."""
    if sniffed.format == "pdf":
        return _unlock_pdf(path, passwords, decrypted_out)
    if sniffed.encrypted_ooxml:
        return _unlock_ooxml(path, passwords, decrypted_out)
    return Unlocked(path, encrypted=False, unlocked_with=None)


def _write_copy(data: bytes, decrypted_out: str | None) -> None:
    if decrypted_out:
        with open(decrypted_out, "wb") as handle:
            handle.write(data)


def _open_pdf(path: str) -> pymupdf.Document:
    try:
        return pymupdf.open(path)
    except Exception as error:  # pymupdf raises several types for damaged files
        raise OpenError("corrupt_file") from error


def _unlock_pdf(path: str, passwords: list[str], decrypted_out: str | None) -> Unlocked:
    probe = _open_pdf(path)
    needs_password = probe.needs_pass
    probe.close()
    if not needs_password:
        return Unlocked(path, encrypted=False, unlocked_with=None)
    for index, candidate in password_attempts(passwords):
        # A fresh handle per attempt: failed tries can leave a handle stuck unauthenticated.
        document = _open_pdf(path)
        try:
            if document.authenticate(candidate):
                data = document.tobytes(encryption=pymupdf.PDF_ENCRYPT_NONE)
                _write_copy(data, decrypted_out)
                return Unlocked(data, encrypted=True, unlocked_with=index)
        finally:
            document.close()
    return Unlocked(path, encrypted=True, unlocked_with=None, locked=True)


def _unlock_ooxml(path: str, passwords: list[str], decrypted_out: str | None) -> Unlocked:
    for index, candidate in password_attempts(passwords):
        try:
            with open(path, "rb") as handle:
                office = msoffcrypto.OfficeFile(handle)  # fresh object per attempt
                # Some issuers' files carry a password verifier that fails for the right
                # password, so the decrypted bytes being a readable zip is the real test.
                office.load_key(password=candidate, verify_password=False)
                buffer = io.BytesIO()
                office.decrypt(buffer)
        except InvalidKeyError:
            continue
        except Exception as error:
            raise OpenError("corrupt_file") from error
        data = buffer.getvalue()
        if not _is_zip(data):
            continue
        _write_copy(data, decrypted_out)
        return Unlocked(data, encrypted=True, unlocked_with=index)
    return Unlocked(path, encrypted=True, unlocked_with=None, locked=True)


def _is_zip(data: bytes) -> bool:
    try:
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            return archive.testzip() is None
    except zipfile.BadZipFile:
        return False
