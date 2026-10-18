"""Runs the helper over every real file in fixtures/private/statements and compares with
fixtures/private/expected/<file>.json. Prints a pass/fail table of counts only.

    uv run python -m tests.golden.run_golden        (or: make finance-golden)
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from statement_reader.contract import CONTRACT_VERSION

from .compare import compare

ROOT = Path(__file__).resolve().parents[2]
PRIVATE = ROOT / "fixtures" / "private"
TIMEOUT_SECONDS = 180


def flat_passwords() -> list[str]:
    """Every saved password, flattened across the groups in passwords.json (order kept)."""
    path = PRIVATE / "passwords.json"
    if not path.exists():
        return []
    groups = json.loads(path.read_text())
    return [p for group in groups.values() for p in group]


def run_helper(file: Path, passwords: list[str], scratch: Path) -> tuple[int, dict | None]:
    request = {"version": CONTRACT_VERSION, "file_path": str(file), "passwords": passwords,
               "decrypted_out": str(scratch / "decrypted")}
    done = subprocess.run([sys.executable, "-m", "statement_reader.cli"], input=json.dumps(request),
                          capture_output=True, text=True, cwd=ROOT, timeout=TIMEOUT_SECONDS,
                          check=False)
    return done.returncode, (json.loads(done.stdout) if done.returncode == 0 else None)


def check_file(file: Path, passwords: list[str], scratch: Path) -> tuple[list[str], dict | None, float]:
    expected_path = PRIVATE / "expected" / f"{file.name}.json"
    if not expected_path.exists():
        return ["no expected file"], None, 0.0
    started = time.perf_counter()
    try:
        code, result = run_helper(file, passwords, scratch)
    except subprocess.TimeoutExpired:
        return [f"timeout after {TIMEOUT_SECONDS}s"], None, time.perf_counter() - started
    seconds = time.perf_counter() - started
    if result is None:
        return [f"crashed (exit {code})"], None, seconds
    return compare(json.loads(expected_path.read_text()), result), result, seconds


def main() -> int:
    files = sorted(p for p in (PRIVATE / "statements").iterdir() if p.is_file())
    if not files:
        print("no files in fixtures/private/statements")
        return 1
    passwords = flat_passwords()
    print(f"{'#':>3}  {'outcome':<11} {'stmts':>5} {'txns':>5}  {'secs':>5}  result  file")
    failed = 0
    with tempfile.TemporaryDirectory() as scratch:
        for number, file in enumerate(files, start=1):
            failures, result, seconds = check_file(file, passwords, Path(scratch))
            outcome = result["outcome"] if result else "-"
            statements = len(result["statements"]) if result else 0
            txns = sum(len(s["transactions"]) for s in result["statements"]) if result else 0
            failed += bool(failures)
            print(f"{number:>3}  {outcome:<11} {statements:>5} {txns:>5}  {seconds:>5.1f}  "
                  f"{'FAIL' if failures else 'ok':<6}  {file.name}")
            for failure in failures[:8]:
                print(f"       - {failure}")
    print(f"\n{len(files) - failed}/{len(files)} files pass")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
