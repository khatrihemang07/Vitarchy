"""Where an opened file's bytes live: its path, or decrypted bytes held in memory."""

from __future__ import annotations

import io
from typing import BinaryIO

Source = str | bytes


def read_bytes(source: Source) -> bytes:
    if isinstance(source, bytes):
        return source
    with open(source, "rb") as handle:
        return handle.read()


def open_stream(source: Source) -> BinaryIO:
    return io.BytesIO(read_bytes(source))
