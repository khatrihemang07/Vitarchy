#!/usr/bin/env python3
"""Scans files for entries of the private denylist: by default every file the next commit could include (tracked, changed or new), or with `--commits A..B` what each of those commits contains.

    scripts/check-private-leaks.sh        (wraps this; run by `make check`)

The denylist (crates/helpers/statement-reader/fixtures/private/denylist.txt, built by
tests/privacy/build_denylist.py) holds identifiers from the person's real Statements. A hit is
reported as file:line and the entry's INDEX (its line number in the denylist), never the entry,
so this output is safe to paste anywhere. Exit 1 on any hit.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DENYLIST = ROOT / "crates/helpers/statement-reader/fixtures/private/denylist.txt"
SKIP_SUFFIXES = (".lock", "pnpm-lock.yaml", "schema.d.ts", ".png", ".jpg", ".ico", ".woff2", ".pdf", ".xls")
X_RUN = r"(?<![A-Za-z])[Xx*•]{2,}\s?"  # two or more, so "px 34" and "0x34" never match


class Denylist:
    def __init__(self, lines: list[str]) -> None:
        self.digit_runs: dict[int, dict[str, int]] = defaultdict(dict)  # length -> {digits: index}
        self.literals: list[tuple[str, int]] = []
        self.words: dict[str, int] = {}
        self.tails: list[tuple[re.Pattern[str], int]] = []
        for index, line in enumerate(lines, start=1):
            kind, _, value = line.partition("\t")
            if not value:
                continue
            if kind == "sub" and value.isdigit():
                self.digit_runs[len(value)][value] = index
            elif kind == "sub":
                self.literals.append((value.lower(), index))
            elif kind == "word":
                self.words[value.lower()] = index
            elif kind == "tail":
                self.tails.append((tail_pattern(value), index))

    def hits(self, line: str) -> set[int]:
        found: set[int] = set()
        lowered = line.lower()
        found.update(index for literal, index in self.literals if literal in lowered)
        for run in re.findall(r"\d{4,}", line):
            for size, entries in self.digit_runs.items():
                for start in range(len(run) - size + 1):
                    index = entries.get(run[start:start + size])
                    if index:
                        found.add(index)
        found.update(self.words[w] for w in re.findall(r"[a-z]+", lowered) if w in self.words)
        found.update(index for pattern, index in self.tails if pattern.search(line))
        return found


def tail_pattern(digits: str) -> re.Pattern[str]:
    """A tail matches after an X-run, or (4 digits only) as a whole token."""
    after_mask = rf"{X_RUN}{digits}(?!\d)"
    if len(digits) == 4:
        return re.compile(rf"{after_mask}|(?<![\w.:-]){digits}(?![\w/])")
    return re.compile(after_mask)


def run_git(*args: str) -> bytes:
    """Runs git in the repo; a failure stops the scan, since a silent empty result would pass a leak."""
    done = subprocess.run(["git", *args], cwd=ROOT, capture_output=True)
    if done.returncode != 0:
        raise SystemExit(f"check-private-leaks: git {args[0]} failed (exit {done.returncode})")
    return done.stdout


def git(*args: str) -> list[str]:
    return [name for name in run_git(*args).decode().split("\n") if name]


def working_files() -> list[tuple[str, bytes]]:
    """Every file that could be committed: tracked, changed, and new but not ignored.

    New files count too: a file not yet `git add`ed is exactly what the next commit picks up.
    """
    names = set(git("ls-files", "--cached")) | set(git("diff", "--name-only"))
    names |= set(git("ls-files", "--others", "--exclude-standard"))
    files = []
    for name in sorted(names):
        path = ROOT / name
        if not name.endswith(SKIP_SUFFIXES) and path.is_file():
            files.append((name, path.read_bytes()))
    return files


def committed_files(commits: str) -> list[tuple[str, bytes]]:
    """The files each commit in `commits` (e.g. origin/main..HEAD) added or changed, as committed.

    Root commits and merge commits count (`--root`, `-m`).
    """
    files = []
    for rev in git("rev-list", commits):
        names = git("diff-tree", "--root", "-m", "--no-commit-id", "--name-only", "-r", "--diff-filter=AM", rev)
        for name in sorted(set(names)):
            if not name.endswith(SKIP_SUFFIXES):
                files.append((f"{rev[:7]}:{name}", run_git("show", f"{rev}:{name}")))
    return files


def main() -> int:
    if not DENYLIST.exists():
        print("check-private-leaks: no denylist (no private data here); skipping")
        return 0
    denylist = Denylist(DENYLIST.read_text().splitlines())
    parser = argparse.ArgumentParser(description="Scan files for private denylist entries.")
    parser.add_argument("--commits", metavar="RANGE", help="scan what each commit in RANGE contains, not the working tree")
    args = parser.parse_args()
    files = committed_files(args.commits) if args.commits else working_files()
    leaks = 0
    for name, data in files:
        if b"\0" in data:
            continue
        for number, line in enumerate(data.decode("utf-8", errors="ignore").splitlines(), start=1):
            for index in sorted(denylist.hits(line)):
                print(f"{name}:{number}: denylist entry #{index}")
                leaks += 1
    if leaks:
        print(f"check-private-leaks: {leaks} hit(s); replace them with synthetic values", file=sys.stderr)
        return 1
    print("check-private-leaks: clean")
    return 0


if __name__ == "__main__":
    sys.exit(main())
