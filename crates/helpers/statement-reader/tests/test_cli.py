import json
import subprocess
import sys

from .conftest import FILES


def run_cli(request: dict | str):
    text = request if isinstance(request, str) else json.dumps({"version": 2, **request})
    done = subprocess.run([sys.executable, "-m", "statement_reader.cli"], input=text,
                          capture_output=True, text=True, check=False)
    return done


def result_of(request: dict) -> tuple[dict, list[dict]]:
    done = run_cli(request)
    assert done.returncode == 0, done.stderr
    return json.loads(done.stdout), [json.loads(line) for line in done.stderr.splitlines()]


def test_a_readable_pdf_reports_its_statement_and_counts():
    result, _ = result_of({"file_path": str(FILES / "ruled_simple.pdf"), "passwords": []})
    assert (result["outcome"], result["kind"], result["format"]) == ("read", "statement", "pdf")
    assert result["diagnostics"] == {"stage": None, "code": None, "pages": 1, "rows": 12}
    assert result["encrypted"] is False and result["unlocked_with"] is None


def test_a_pdf_with_no_table_reports_the_locate_stage(tmp_path):
    import pymupdf

    document = pymupdf.open()
    document.new_page().insert_text((50, 50), "just some prose, no table")
    path = tmp_path / "prose.pdf"
    document.save(path)
    result, _ = result_of({"file_path": str(path), "passwords": []})
    assert result["outcome"] == "no_table"
    assert result["diagnostics"] == {"stage": "locate", "code": "no_header_or_columns", "pages": 1, "rows": None}


def test_a_spreadsheet_reports_its_row_count():
    result, _ = result_of({"file_path": str(FILES / "ghost_row.xls")})
    assert (result["format"], result["outcome"]) == ("xls", "read")
    assert result["diagnostics"]["rows"] == 8


def test_an_encrypted_pdf_unlocks_and_writes_the_decrypted_copy(tmp_path):
    out = tmp_path / "copy.pdf"
    result, _ = result_of({"file_path": str(FILES / "encrypted.pdf"),
                           "passwords": ["nope", "synth1234"], "decrypted_out": str(out)})
    assert (result["encrypted"], result["unlocked_with"], result["format"]) == (True, 1, "pdf")
    assert out.exists()


def test_a_file_no_password_opens_is_locked(tmp_path):
    out = tmp_path / "copy.xlsx"
    result, _ = result_of({"file_path": str(FILES / "encrypted_sheet.xlsx"),
                           "passwords": ["nope"], "decrypted_out": str(out)})
    assert result["outcome"] == "locked" and result["encrypted"] is True
    assert result["diagnostics"]["stage"] == "open" and not out.exists()


def test_an_unrecognised_file_is_unsupported(tmp_path):
    path = tmp_path / "x.bin"
    path.write_bytes(b"\x00\x01\x02")
    result, _ = result_of({"file_path": str(path)})
    assert result["outcome"] == "unsupported" and result["format"] is None


def test_a_corrupt_pdf_is_unsupported_not_a_crash(tmp_path):
    path = tmp_path / "x.pdf"
    path.write_bytes(b"%PDF-1.4\nnot really a pdf")
    result, _ = result_of({"file_path": str(path)})
    assert result["outcome"] == "unsupported" and result["diagnostics"]["code"] == "corrupt_file"


def test_stderr_holds_only_stage_names_counts_and_durations():
    password = "synth1234"
    _, lines = result_of({"file_path": str(FILES / "encrypted.pdf"), "passwords": [password]})
    stages = [line["stage"] for line in lines if line["event"] == "stage"]
    assert stages == ["sniff", "unlock", "read", "locate", "engine"]
    assert all("duration_ms" in line for line in lines if line["event"] == "stage")
    raw = json.dumps(lines)
    assert password not in raw and "Example Bank" not in raw and str(FILES) not in raw


def test_a_bad_request_is_a_crash_with_a_nonzero_exit():
    assert run_cli("not json").returncode != 0
    assert run_cli(json.dumps({"version": 9, "file_path": "x"})).returncode != 0


def test_a_missing_file_is_a_crash():
    assert run_cli({"file_path": "/nonexistent/file.pdf"}).returncode != 0


def test_the_request_id_from_the_environment_tags_every_log_line(monkeypatch):
    monkeypatch.setenv("STATEMENT_READER_REQUEST_ID", "req-42")
    _, lines = result_of({"file_path": str(FILES / "ruled_simple.pdf")})
    assert lines and all(line["request_id"] == "req-42" for line in lines)
