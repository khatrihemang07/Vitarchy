"""Builds fixtures/private/denylist.txt: every identifier in the private real Statements.

    uv run python -m tests.privacy.build_denylist

`scripts/check-private-leaks.sh` scans the committed tree for these entries so real account
numbers, references, names and merchants never reach git. The denylist is gitignored with the rest
of fixtures/private and this script holds no data. Line format: `kind<TAB>value`, where kind is
`sub` (case-insensitive substring), `word` (case-insensitive whole word) or `tail` (a 2 or 4 digit
tail, matched whole or after an X-run).
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

from statement_reader.contract import CONTRACT_VERSION

HELPER = Path(__file__).resolve().parents[2]
PRIVATE = HELPER / "fixtures" / "private"
OUT = PRIVATE / "denylist.txt"
MIN_WORD = 5
# Generic words that real descriptions and labels are full of; never worth flagging.
STOPLIST = set(["transfer", "payment", "payments", "credit", "debit", "account", "statement", "balance", "amount", "closing", "opening", "interest", "charges", "charge", "purchase", "purchases", "refund", "reversal", "withdrawal", "deposit", "cheque", "number", "customer", "address", "branch", "period", "total", "summary", "details", "limit", "available", "outstanding", "minimum", "online", "mobile", "banking", "internet", "instant", "immediate", "message", "limited", "private", "public", "india", "indian", "bank", "card", "cards", "cash", "received", "paid", "fees", "finance", "service", "services", "between", "rewards", "points", "domestic", "international", "payee", "reference", "transaction", "transactions", "date", "dated"])
# Public names and PDF metadata that are safe in tests: issuers, apps, fonts, cities.
PUBLIC = set(["phonepe", "amazon", "mumbai", "bangalore", "centre", "helvetica", "arial", "moddate", "creationdate", "producer", "statebank", "commerc", "ebcdf", "box", "encoding", "requested", "dns", "groceries"])
DICTIONARY = Path("/usr/share/dict/words")
NAME_LABEL = re.compile(r"\b(?:name|mr|mrs|ms|miss|shri|smt|dear|holder|customer|nominee)\b[\s.:]*", re.I)
NAME_NOISE = set(["the", "and", "for", "with", "from", "not", "name", "statement", "account", "holder", "customer"])
HEADER_LINES = 40  # the first lines of a page hold the holder's name and address


def vocabulary_words() -> set[str]:
    """Every word in vocabulary.toml: bank terms that are safe to appear anywhere."""
    data = tomllib.loads((HELPER / "statement_reader" / "vocabulary.toml").read_text())
    words: set[str] = set()

    def walk(node) -> None:
        if isinstance(node, str):
            words.update(re.findall(r"[a-z]+", node.lower()))
        elif isinstance(node, dict):
            for key, value in node.items():
                words.update(re.findall(r"[a-z]+", str(key).lower()))
                walk(value)
        elif isinstance(node, list):
            for value in node:
                walk(value)

    walk(data)
    return words


def english_words() -> set[str]:
    if not DICTIONARY.exists():
        return set()
    return {w.strip().lower() for w in DICTIONARY.read_text(errors="ignore").splitlines() if w.islower()}


# The last commit before Finance existed. What the person had already chosen to commit by then
# (their own display name in samples, ports, generic words) is theirs, not a leak from a Statement.
# Pinned rather than HEAD, so later Finance commits can never whitelist their own leaks.
BASELINE = "93d2320"


def baseline_tokens() -> set[str]:
    """Lower-cased word and digit tokens in the pre-Finance tree."""
    root = HELPER.parents[2]
    names = subprocess.run(["git", "ls-tree", "-r", "--name-only", BASELINE], cwd=root, capture_output=True,
                           text=True, check=True).stdout.splitlines()
    tokens: set[str] = set()
    for name in names:
        if name.endswith((".lock", ".png", ".ico", ".woff2", ".pdf", "schema.d.ts")):
            continue
        blob = subprocess.run(["git", "show", f"{BASELINE}:{name}"], cwd=root, capture_output=True).stdout
        tokens.update(re.findall(r"[a-z]+|\d+", blob.decode("utf-8", errors="ignore").lower()))
    return tokens


def flatten_passwords() -> list[str]:
    groups = json.loads((PRIVATE / "passwords.json").read_text())
    return [p for group in groups.values() for p in group]


def run_helper(path: Path, passwords: list[str], scratch: Path) -> tuple[dict, Path]:
    request = {"version": CONTRACT_VERSION, "file_path": str(path), "passwords": passwords,
               "decrypted_out": str(scratch / "decrypted")}
    done = subprocess.run([sys.executable, "-m", "statement_reader.cli"], input=json.dumps(request),
                          capture_output=True, text=True, cwd=HELPER, check=True, timeout=300)
    decrypted = scratch / "decrypted"
    return json.loads(done.stdout), (decrypted if decrypted.exists() else path)


def page_texts(path: Path, passwords: list[str]) -> list[str]:
    """Raw text per page (PDF) or per sheet row (spreadsheets, CSV), to find names and addresses."""
    suffix = path.suffix.lower()
    if suffix == ".pdf":
        import fitz

        doc = fitz.open(path)
        if doc.needs_pass:
            for password in passwords:
                for variant in (password, password.upper(), password.lower(), password.capitalize()):
                    if doc.authenticate(variant):
                        break
                else:
                    continue
                break
        return [page.get_text() for page in doc]
    if suffix == ".xlsx":
        import openpyxl

        book = openpyxl.load_workbook(path, read_only=True, data_only=True)
        return ["\n".join(" ".join(str(c) for c in row if c is not None) for row in sheet.iter_rows(values_only=True))
                for sheet in book]
    if suffix == ".xls":
        import xlrd

        book = xlrd.open_workbook(path)
        return ["\n".join(" ".join(str(c) for c in sheet.row_values(i) if c != "") for i in range(sheet.nrows))
                for sheet in book.sheets()]
    return [path.read_text(errors="ignore")]


def words_of(text: str) -> list[str]:
    return [w.lower() for w in re.findall(rf"[A-Za-z]{{{MIN_WORD},}}", text)]


class Collector:
    def __init__(self) -> None:
        self.sub: set[str] = set()
        self.word: set[str] = set()
        self.tail: set[str] = set()
        self.safe = STOPLIST | PUBLIC | vocabulary_words()
        self.names: set[str] = set()
        self.english = english_words()

    def is_english(self, word: str) -> bool:
        return word in self.english or word.rstrip("s") in self.english or word.rstrip("es") in self.english

    def digits(self, text: str) -> None:
        """Long numbers: references, account numbers, phones."""
        for run in re.findall(r"\d{9,}|(?:\d{4}[ -]){2,}\d{1,4}", text):
            compact = re.sub(r"[\s-]", "", run)
            if len(compact) >= 9 and len(set(compact)) >= 4:  # "0000000000" is filler, not an identifier
                self.sub.add(compact)

    def mask(self, mask: str | None) -> None:
        if not mask:
            return
        compact = mask.replace(" ", "")
        if len(compact) >= 9:
            self.sub.update({mask, compact})
        match = re.search(r"\d+$", mask.strip())
        if match:
            digits = match.group(0)
            if len(digits) >= 9:
                self.sub.add(digits)
            for size in (2, 4):
                if len(digits) >= size:
                    self.tail.add(digits[-size:])

    def holder(self, text: str) -> None:
        """Name, address, email and phone tokens from the lines that open a page."""
        for email in re.findall(r"[A-Za-z0-9._+-]{3,}@[A-Za-z0-9-]{3,}(?:\.[A-Za-z]{2,})+", text):
            self.sub.add(email.lower())
        for phone in re.findall(r"(?<!\d)[6-9]\d{9}(?!\d)", text):
            self.sub.add(phone)
        for line in text.splitlines()[:HEADER_LINES]:
            if re.match(r"\s*\d{1,2}[/.-]\d{1,2}[/.-]\d{2,4}", line):
                break
            for word in words_of(line):
                if word not in self.safe and not self.is_english(word):
                    self.word.add(word)
        for match in NAME_LABEL.finditer(text):
            tail = text[match.end(): match.end() + 60].splitlines()[0] if text[match.end():] else ""
            for word in re.findall(r"[A-Za-z]{3,}", tail):
                if word.lower() not in self.safe and word.lower() not in NAME_NOISE and not self.is_english(word.lower()):
                    self.names.add(word.lower())

    def free_text(self, text: str) -> None:
        """Descriptions and counterparties: the words that are not ordinary English or bank terms."""
        self.digits(text)
        for word in words_of(text):
            if word not in self.safe and not self.is_english(word):
                self.word.add(word)

    def result(self, result: dict) -> None:
        for statement in result.get("statements", []):
            self.mask(statement["account"].get("mask"))
            for row in statement.get("transactions", []):
                for key in ("description", "reference", "utr", "source_id"):
                    if row.get(key):
                        self.free_text(str(row[key]))

    def lines(self) -> list[str]:
        baseline = baseline_tokens()
        dropped = self.safe | baseline
        words = sorted({w for w in self.word | self.names if w not in dropped and not set(w) <= {"x"}})
        return ([f"sub\t{v}" for v in sorted(self.sub) if v.lower() not in baseline]
                + [f"word\t{v}" for v in words]
                + [f"tail\t{v}" for v in sorted(self.tail) if v not in baseline])


def main() -> int:
    passwords = flatten_passwords()
    collector = Collector()
    files = sorted(p for p in (PRIVATE / "statements").iterdir() if p.is_file())
    with tempfile.TemporaryDirectory() as tmp:
        for index, path in enumerate(files):
            scratch = Path(tmp) / str(index)
            scratch.mkdir()
            for printed in re.findall(r"\d{4}X+\d{4}", path.name):
                collector.mask(printed)  # card masks that file names carry
            result, readable = run_helper(path, passwords, scratch)
            collector.result(result)
            for text in page_texts(readable, passwords):
                collector.digits(text)
                collector.holder(text)
    lines = collector.lines()
    OUT.write_text("\n".join(lines) + "\n")
    print(f"wrote {len(lines)} denylist entries from {len(files)} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
