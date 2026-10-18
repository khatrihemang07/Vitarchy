import pymupdf
import pytest

from statement_reader.open.sniff import sniff
from statement_reader.open.unlock import password_attempts, unlock

from .conftest import FILES


def test_attempts_try_each_password_with_its_case_variants_once():
    assert password_attempts(["abC1"]) == [(0, "abC1"), (0, "ABC1"), (0, "abc1"), (0, "Abc1")]


def test_attempts_deduplicate_across_passwords_and_keep_the_first_index():
    attempts = password_attempts(["abc", "ABC", "other", ""])
    assert attempts == [(0, "abc"), (0, "ABC"), (0, "Abc"), (2, "other"), (2, "OTHER"), (2, "Other")]


def run(name, passwords, out=None):
    path = str(FILES / name)
    return unlock(path, sniff(path), passwords, out)


def test_a_plain_file_is_not_encrypted_and_writes_no_copy(tmp_path):
    out = tmp_path / "copy.pdf"
    result = run("ruled_simple.pdf", ["whatever"], str(out))
    assert (result.encrypted, result.unlocked_with, result.locked) == (False, None, False)
    assert not out.exists()


def test_a_pdf_opens_with_the_capitalised_variant_and_reports_its_index(tmp_path):
    out = tmp_path / "copy.pdf"
    result = run("encrypted.pdf", ["wrong", "synth1234", "later"], str(out))
    assert (result.encrypted, result.unlocked_with, result.locked) == (True, 1, False)
    copy = pymupdf.open(out)
    assert not copy.needs_pass and copy.page_count == 1
    assert pymupdf.open(stream=result.source, filetype="pdf").page_count == 1


def test_many_failed_attempts_do_not_stick_a_later_correct_password():
    wrong = [f"nope{i}" for i in range(15)]
    assert run("encrypted.pdf", [*wrong, "synth1234"]).unlocked_with == 15


def test_a_pdf_with_no_matching_password_is_locked_and_writes_nothing(tmp_path):
    out = tmp_path / "copy.pdf"
    result = run("encrypted.pdf", ["nope", "also-no"], str(out))
    assert (result.encrypted, result.unlocked_with, result.locked) == (True, None, True)
    assert not out.exists()
    assert run("encrypted.pdf", []).locked


def test_an_encrypted_workbook_opens_with_the_upper_case_variant(tmp_path):
    out = tmp_path / "copy.xlsx"
    result = run("encrypted_sheet.xlsx", ["nope", "xlsx2026"], str(out))
    assert (result.encrypted, result.unlocked_with) == (True, 1)
    import openpyxl

    assert openpyxl.load_workbook(out).active["A7"].value == "Date"


def test_an_encrypted_workbook_with_no_matching_password_is_locked(tmp_path):
    out = tmp_path / "copy.xlsx"
    result = run("encrypted_sheet.xlsx", ["nope"], str(out))
    assert result.locked and not out.exists()


@pytest.mark.parametrize("name", ["ghost_row.xls", "payment_history.csv"])
def test_formats_that_cannot_be_encrypted_pass_straight_through(name, tmp_path):
    out = tmp_path / "copy"
    assert run(name, ["x"], str(out)).encrypted is False
    assert not out.exists()
