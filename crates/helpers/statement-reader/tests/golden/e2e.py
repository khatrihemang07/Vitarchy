"""End to end over the private real Statements, against a STAGE server. Prints counts only.

Run through `make finance-e2e`, which supplies the stage environment and the built server:

    scripts/finance-e2e.sh

It creates a fresh `vitarchy_e2e` database on the stage Postgres, starts the built server on a
free port with this helper as the reader, drives the whole import flow and stops the server by
PID. It refuses to run if the stage database is the live one.

Four scenarios, each on a fresh database: the normal import in two orders (the PhonePe files
confirmed last, then first; Statements confirm with "use the match", PhonePe sections with their
proposal or the private expectations' Account for the instrument), a page-style import (confirm
with the choices the web page builds from the previews, one section at a time), and a repair
(duplicate Accounts made by bypassing the server's rule, then merged by
`vitarchy-server finance-merge-duplicates`). Both orders must leave the same Transactions per
Account, and the last two the same counts as the PhonePe-last import.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from urllib.parse import urlparse

from e2e_assertions import compare_orders
from e2e_env import E2E_DB, PRIVATE, Api, failures, free_port, load_expected, psql, refuse_live, start_server
from e2e_normal import run
from e2e_page_style import run_page_style
from e2e_repair import run_repair


def main() -> int:
    stage_url = os.environ.get("DATABASE_URL", "")
    if not stage_url or not os.environ.get("VITARCHY_STAGE"):
        sys.exit("run through scripts/finance-e2e.sh (stage environment)")
    refuse_live(stage_url)
    expected = load_expected()
    if not (PRIVATE / "statements").is_dir() or not (PRIVATE / "passwords.json").exists():
        sys.exit("the private fixtures are missing")
    admin_db = urlparse(stage_url).path.lstrip("/")
    scratch = tempfile.mkdtemp(prefix="vitarchy-e2e-")
    normal: dict[str, int] = {}
    orders: dict[str, dict] = {}

    def run_order(phonepe: str):
        def go(api: Api) -> None:
            print(f"normal import, PhonePe {phonepe}:")
            orders[phonepe] = run(api, stage_url, expected, phonepe)
            if phonepe == "last":
                normal.update(orders[phonepe]["counts"])
        return go

    scenarios = [("phonepe-last", run_order("last")), ("phonepe-first", run_order("first")),
                 ("page-style", lambda api: run_page_style(api, stage_url, normal, expected)),
                 ("repair", lambda api: run_repair(api, stage_url, normal, expected))]
    for name, scenario in scenarios:
        psql(stage_url, admin_db, f"DROP DATABASE IF EXISTS {E2E_DB} WITH (FORCE)")
        psql(stage_url, admin_db, f"CREATE DATABASE {E2E_DB}")
        log = Path(scratch) / f"server-{name}.log"
        port = free_port()
        server = start_server(stage_url, port, str(Path(scratch) / "statements"), log)
        try:
            scenario(Api(port))
        finally:
            server.terminate()
            try:
                server.wait(timeout=15)
            except subprocess.TimeoutExpired:
                server.kill()
    compare_orders(orders)
    if not failures:
        shutil.rmtree(scratch, ignore_errors=True)
    print(f"{'FAILED' if failures else 'OK'}: {len(failures)} failed assertion(s)" + (f"; server logs in {scratch}" if failures else ""))
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
