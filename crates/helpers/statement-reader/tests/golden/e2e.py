"""End to end over the private real Statements, against a STAGE server. Prints counts only.

Run through `make finance-e2e`, which supplies the stage environment and the built server:

    scripts/finance-e2e.sh

It creates a fresh `vitarchy_e2e` database on the stage Postgres, starts the built server on a
free port with this helper as the reader, drives the whole import flow and stops the server by
PID. It refuses to run if the stage database is the live one.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import socket
import subprocess
import sys
import tempfile
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


def visible_digits(mask: str | None) -> str:
    match = re.search(r"\d+$", (mask or "").strip())
    return match.group(0) if match else ""


def same_number(a: str | None, b: str | None) -> bool:
    da, db = visible_digits(a), visible_digits(b)
    if len(da) >= 2 and len(db) >= 2:
        n = min(len(da), len(db))
        return da[-n:] == db[-n:]
    return bool(a) and "".join((a or "").upper().split()) == "".join((b or "").upper().split())


def confirm_all(api: Api, previews: list[dict]) -> dict[str, int]:
    """Confirms every file (Statements first); returns what the previews promised and what the
    confirms did, so the two can be compared."""
    refused = 0
    totals = {"previewed_gain": 0, "confirmed_gain": 0}
    for preview in sorted(previews, key=lambda p: p["kind"] == "payment_history"):
        sections = []
        for section in preview.get("sections") or []:
            proposed = section["proposed"]
            if proposed.get("issuer") and proposed.get("kind"):
                sections.append({})
            else:
                sections.append({"new_account": {"issuer": proposed.get("issuer") or "Unknown",
                                                 "kind": proposed.get("kind") or "savings", "name": "Unnamed"}})
        # The preview as it is just before this confirm, against what earlier confirms stored.
        _, current = api.call("GET", f"/api/finance/statements/{preview['id']}")
        totals["previewed_gain"] += sum(s["enriched_count"] for s in (current or {}).get("sections", []))
        status, reply = api.call("POST", f"/api/finance/statements/{preview['id']}/confirm", {"sections": sections})
        if status != 200:
            check(False, f"confirm statement {preview['id']} returned 200 (got {status}, code {(reply or {}).get('code')})")
            refused += 1
        else:
            totals["confirmed_gain"] += reply["enriched"]
    check(refused == 0, f"every confirm returned 200 ({len(previews) - refused}/{len(previews)})")
    return totals


def assert_accounts(accounts: list[dict], expected: dict) -> None:
    expected_accounts = [tuple(a) for a in expected["accounts"]]
    unique = {tuple(u[:3]): u[3] for u in expected["unique_transactions"]}
    sbi_loans = expected["sbi_loans"]
    check(len(accounts) == len(expected_accounts) + sbi_loans, f"{len(expected_accounts) + sbi_loans} Accounts (got {len(accounts)})")
    for issuer, kind, digits in expected_accounts:
        found = [a for a in accounts if issuer in a["issuer"].lower() and a["kind"] == kind
                 and same_number(a["mask"], digits)]
        check(len(found) == 1, f"one {issuer} {kind} Account ending {digits} (got {len(found)})")
        if (issuer, kind, digits) in unique and found:
            want = unique[(issuer, kind, digits)]
            check(found[0]["transaction_count"] == want, f"{issuer} {kind} {digits}: {want} Transactions (got {found[0]['transaction_count']})")
    loans = [a for a in accounts if "state bank" in a["issuer"].lower() and a["kind"] == "loan"]
    check(len(loans) == sbi_loans, f"{sbi_loans} SBI loans (got {len(loans)})")


def assert_links(stage_url: str) -> None:
    rows = psql(stage_url, E2E_DB,
                "SELECT coalesce(p.instrument_issuer,''), coalesce(p.instrument_digits,''), a.issuer, coalesce(a.mask,'') "
                "FROM finance.payment_records p JOIN finance.transactions t ON t.id = p.transaction_id "
                "JOIN finance.accounts a ON a.id = t.account_id").splitlines()
    disagree = 0
    for row in rows:
        issuer, digits, account_issuer, mask = row.split("|")
        issuer_ok = not issuer or " ".join(issuer.lower().split()) == " ".join(account_issuer.lower().split())
        digits_ok = not digits or same_number(digits, mask)
        disagree += not (issuer_ok and digits_ok)
    check(disagree == 0, f"no Payment record links to an Account that disagrees with its instrument ({len(rows)} linked)")
    print("  linked per instrument (issuer, ...last 2 digits: linked/total):")
    for line in psql(stage_url, E2E_DB,
                     "SELECT coalesce(instrument_issuer,'?'), right(coalesce(instrument_digits,'??'), 2), "
                     "count(transaction_id), count(*) FROM finance.payment_records GROUP BY 1, 2 ORDER BY 1, 2").splitlines():
        issuer, tail, linked, total = line.split("|")
        print(f"    {issuer} ...{tail}: {linked}/{total}")


def assert_printings(stage_url: str, expected: dict, gains: dict[str, int]) -> None:
    """Counts only: every Transaction keeps its Printings, overlapping SBI files show the fuller text."""
    count = lambda sql: int(psql(stage_url, E2E_DB, sql).strip())  # noqa: E731
    transactions = count("SELECT count(*) FROM finance.transactions")
    printings = count("SELECT count(*) FROM finance.printings")
    orphans = count("SELECT count(*) FROM finance.transactions t WHERE NOT EXISTS "
                    "(SELECT 1 FROM finance.printings p WHERE p.transaction_id = t.id)")
    print(f"  {transactions} Transactions, {printings} Printings, {gains['confirmed_gain']} enriched by confirms")
    check(orphans == 0 and printings >= transactions, f"every Transaction has a Printing ({orphans} without)")
    check(gains["previewed_gain"] > 0, f"previews said some rows would gain detail ({gains['previewed_gain']})")
    check(gains["previewed_gain"] == gains["confirmed_gain"],
          f"confirms enriched exactly what the previews promised ({gains['confirmed_gain']} of {gains['previewed_gain']})")
    both = expected["printed_by_pdf_and_xlsx"]
    sql = ("SELECT count(*), count(*) FILTER (WHERE t.description = (SELECT p.description FROM finance.printings p "
           "JOIN finance.statements s ON s.id = p.statement_id WHERE p.transaction_id = t.id AND s.format = 'xlsx' ORDER BY length(p.description) DESC LIMIT 1)) "
           "FROM finance.transactions t JOIN finance.accounts a ON a.id = t.account_id "
           f"WHERE a.issuer ILIKE '%{both['issuer']}%' AND a.kind = '{both['kind']}' "
           "AND (SELECT count(*) FROM finance.printings p WHERE p.transaction_id = t.id) >= 2 "
           "AND EXISTS (SELECT 1 FROM finance.printings p JOIN finance.statements s ON s.id = p.statement_id "
           "WHERE p.transaction_id = t.id AND s.format = 'pdf') "
           "AND EXISTS (SELECT 1 FROM finance.printings p JOIN finance.statements s ON s.id = p.statement_id "
           "WHERE p.transaction_id = t.id AND s.format = 'xlsx')")
    printed_both, show_xlsx = (int(n) for n in psql(stage_url, E2E_DB, sql).strip().split("|"))
    print(f"  {printed_both} printed by a PDF and an XLSX, {show_xlsx} show the XLSX's text")
    check(printed_both == both["count"], f"{both['count']} {both['issuer']} {both['kind']} Transactions have a PDF Printing and an XLSX Printing (got {printed_both})")
    # Informational, not asserted: the PDF's narration is often not a strict prefix of the
    # XLSX's (its last letter differs), so the strict-extension rule keeps the PDF text there.
    print(f"  INFO  {show_xlsx} of {printed_both} show the XLSX's fuller text")


def run(api: Api, stage_url: str, expected: dict) -> None:
    passwords = json.loads((PRIVATE / "passwords.json").read_text())
    for label, secrets in passwords.items():
        for index, secret in enumerate(secrets):
            status, _ = api.call("POST", "/api/finance/statement-passwords",
                                 {"label": f"{label.split('_format')[0]}-{index}", "secret": secret})
            if status not in (200, 201):
                check(False, f"saved password {label.split('_format')[0]}-{index} (got {status})")
    files = sorted(p for p in (PRIVATE / "statements").iterdir() if p.is_file())
    status, previews = api.upload(files)
    print(f"uploaded {len(files)} files, status {status}")
    check(len(previews) == FILE_COUNT and all(p["outcome"] == "read" for p in previews),
          f"all {FILE_COUNT} files read ({sum(p['outcome'] == 'read' for p in previews)}/{len(previews)})")
    failed = [(s["proposed"].get("issuer") or "") for p in previews for s in p.get("sections") or []
              for c in s["checks"] if c["outcome"] == "failed"]
    issuer = expected["failed_check"]["issuer"]
    check(len(failed) == 1 and issuer in failed[0].lower(), f"exactly 1 failed Check, in the expected file (got {len(failed)})")
    gains = confirm_all(api, previews)
    assert_accounts(api.call("GET", "/api/finance/accounts")[1], expected)
    assert_links(stage_url)
    assert_printings(stage_url, expected, gains)
    _, again = api.upload(files)
    dups = sum(1 for p in again if p.get("duplicate_file"))
    check(dups == FILE_COUNT, f"re-upload: {dups}/{len(again)} duplicate_file")
    adds = vanished = new_printings = enriched = errors = 0
    for preview in previews:
        status, reply = api.call("POST", f"/api/finance/statements/{preview['id']}/read-again")
        errors += status != 200
        for section in (reply or {}).get("sections", []):
            adds += section["add_count"]
            vanished += section["no_longer_found_count"]
            new_printings += section["new_printing_count"]
            enriched += section["enriched_count"]
    check(errors == 0, f"every Read again returned 200 ({len(previews) - errors}/{len(previews)})")
    check(adds == 0 and vanished == 0 and new_printings == 0 and enriched == 0,
          f"Read again: {adds} to add, {new_printings} Printings to add ({enriched} gaining detail), {vanished} no longer found")


def main() -> int:
    stage_url = os.environ.get("DATABASE_URL", "")
    if not stage_url or not os.environ.get("VITARCHY_STAGE"):
        sys.exit("run through scripts/finance-e2e.sh (stage environment)")
    refuse_live(stage_url)
    expected = load_expected()
    if not (PRIVATE / "statements").is_dir() or not (PRIVATE / "passwords.json").exists():
        sys.exit("the private fixtures are missing")
    admin_db = urlparse(stage_url).path.lstrip("/")
    psql(stage_url, admin_db, f"DROP DATABASE IF EXISTS {E2E_DB} WITH (FORCE)")
    psql(stage_url, admin_db, f"CREATE DATABASE {E2E_DB}")
    scratch = tempfile.mkdtemp(prefix="vitarchy-e2e-")
    log = Path(scratch) / "server.log"
    port = free_port()
    server = start_server(stage_url, port, str(Path(scratch) / "statements"), log)
    try:
        run(Api(port), stage_url, expected)
    finally:
        server.terminate()
        try:
            server.wait(timeout=15)
        except subprocess.TimeoutExpired:
            server.kill()
        if not failures:
            shutil.rmtree(scratch, ignore_errors=True)
    print(f"{'FAILED' if failures else 'OK'}: {len(failures)} failed assertion(s)" + (f"; server log in {log}" if failures else ""))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
