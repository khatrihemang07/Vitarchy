"""Shared plumbing of the end-to-end run: paths, the assertion log, the stage database, the server
and its JSON client. Counts only; nothing private is printed."""

from __future__ import annotations

import json
import os
import re
import socket
import subprocess
import sys
import time
import urllib.error
import urllib.request
import uuid
from pathlib import Path
from urllib.parse import urlparse

HELPER = Path(__file__).resolve().parents[2]
ROOT = HELPER.parents[2]
PRIVATE = HELPER / "fixtures" / "private"
E2E_DB = "vitarchy_e2e"
FILE_COUNT = 31

EXPECTED_FILE = PRIVATE / "e2e_expected.json"


def load_expected() -> dict:
    """What the private real Statements must produce. It lives beside them, never in git:
    {"accounts": [[issuer contains, kind, trailing digits shown]...], "sbi_loans": n,
     "unique_transactions": [[issuer, kind, digits, count]...], "failed_check": {"issuer": ...}}."""
    if not EXPECTED_FILE.exists():
        sys.exit(f"missing {EXPECTED_FILE.relative_to(ROOT)}: the real-Statement expectations are private")
    return json.loads(EXPECTED_FILE.read_text())


failures: list[str] = []


def check(ok: bool, what: str) -> None:
    """Records one assertion and prints PASS or FAIL (never any content)."""
    print(f"  {'PASS' if ok else 'FAIL'}  {what}")
    if not ok:
        failures.append(what)


def env_url(path: Path) -> str:
    """DATABASE_URL from a dotenv file, parsed (never sourced)."""
    for line in reversed(path.read_text().splitlines()):
        match = re.match(r"\s*DATABASE_URL\s*=\s*(.*?)\s*$", line)
        if match:
            return match.group(1).strip("'\"")
    sys.exit(f"no DATABASE_URL in {path.name}")


def refuse_live(stage_url: str) -> None:
    live = urlparse(env_url(ROOT / ".env" if (ROOT / ".env").exists() else ROOT / ".env.example"))
    stage = urlparse(stage_url)
    if stage.port == live.port or stage.path == live.path or live.path.lstrip("/") == E2E_DB:
        sys.exit("refusing: the stage database looks like the LIVE one")


def psql(stage_url: str, database: str, sql: str) -> str:
    """Runs SQL on the stage Postgres and returns tuples-only output."""
    url = urlparse(stage_url)
    env = {**os.environ, "PGPASSWORD": url.password or ""}
    done = subprocess.run(
        ["psql", "-h", url.hostname or "127.0.0.1", "-p", str(url.port or 5432), "-U", url.username or "",
         "-d", database, "-At", "-F", "|", "-v", "ON_ERROR_STOP=1", "-c", sql],
        env=env, capture_output=True, text=True, check=True)
    return done.stdout


def free_port() -> int:
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


class Api:
    """Thin JSON client for the Finance API."""

    def __init__(self, port: int) -> None:
        self.base = f"http://127.0.0.1:{port}"

    def call(self, method: str, path: str, body=None, raw=None, ctype="application/json"):
        data = raw if raw is not None else (json.dumps(body).encode() if body is not None else None)
        request = urllib.request.Request(self.base + path, data=data, method=method,
                                         headers={"content-type": ctype} if data else {})
        try:
            with urllib.request.urlopen(request, timeout=900) as response:
                text = response.read()
                return response.status, (json.loads(text) if text else None)
        except urllib.error.HTTPError as error:
            return error.code, json.loads(error.read() or b"null")

    def upload(self, paths: list[Path]):
        boundary = uuid.uuid4().hex
        parts: list[bytes] = []
        for path in paths:
            parts += [f'--{boundary}\r\nContent-Disposition: form-data; name="files"; filename="{path.name}"'
                      "\r\nContent-Type: application/octet-stream\r\n\r\n".encode(), path.read_bytes(), b"\r\n"]
        parts.append(f"--{boundary}--\r\n".encode())
        return self.call("POST", "/api/finance/statements", raw=b"".join(parts),
                         ctype=f"multipart/form-data; boundary={boundary}")


def start_server(stage_url: str, port: int, statements_dir: str, log: Path) -> subprocess.Popen:
    url = urlparse(stage_url)
    e2e_url = url._replace(path=f"/{E2E_DB}").geturl()
    env = {**os.environ, "DATABASE_URL": e2e_url, "BIND_ADDR": f"127.0.0.1:{port}", "WEB_DIST": "",
           "FINANCE_READER": str(HELPER / ".venv" / "bin" / "statement-reader"),
           "FINANCE_STATEMENTS_DIR": statements_dir, "RUST_LOG": "warn", "VITARCHY_STAGE": "1"}
    process = subprocess.Popen([str(ROOT / "target" / "debug" / "vitarchy-server")], env=env,
                               stdout=log.open("w"), stderr=subprocess.STDOUT)
    for _ in range(120):
        if process.poll() is not None:
            sys.exit(f"the server exited early; see {log}")
        try:
            urllib.request.urlopen(f"http://127.0.0.1:{port}/api/health", timeout=2)
            return process
        except OSError:
            time.sleep(0.5)
    process.kill()
    sys.exit(f"the server did not become healthy; see {log}")


def counts(stage_url: str) -> dict[str, int]:
    """Row counts of what a finished import leaves: the figures every scenario must agree on."""
    sql = ("SELECT (SELECT count(*) FROM finance.accounts), (SELECT count(*) FROM finance.transactions), "
           "(SELECT count(*) FROM finance.printings), "
           "(SELECT count(*) FROM finance.instrument_accounts)")
    keys = ("accounts", "transactions", "printings", "instruments")
    return dict(zip(keys, (int(n) for n in psql(stage_url, E2E_DB, sql).strip().split("|")), strict=True))
