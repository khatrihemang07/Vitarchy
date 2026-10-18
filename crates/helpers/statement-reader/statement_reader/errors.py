"""Failures the CLI turns into a Result (never a crash): a file that cannot be opened or read."""


class OpenError(Exception):
    """The file is not a usable PDF/CSV/XLS/XLSX (corrupt or unrecognised): outcome `unsupported`."""

    def __init__(self, code: str) -> None:
        super().__init__(code)
        self.code = code


class ReadError(Exception):
    """The file opened but its pages or cells could not be read: outcome `no_table`, stage `read`."""

    def __init__(self, code: str) -> None:
        super().__init__(code)
        self.code = code
