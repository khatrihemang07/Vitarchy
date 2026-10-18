import pytest

from statement_reader.open.sniff import Sniffed, sniff

from .conftest import FILES, MANIFEST

EXPECTED = {
    "ruled_simple.pdf": Sniffed("pdf"),
    "encrypted.pdf": Sniffed("pdf"),
    "encrypted_sheet.xlsx": Sniffed("xlsx", encrypted_ooxml=True),
    "ghost_row.xls": Sniffed("xls"),
    "payment_history.csv": Sniffed("csv"),
}


@pytest.mark.parametrize("name", MANIFEST)
def test_every_synthetic_file_is_recognised_by_its_bytes(name):
    sniffed = sniff(str(FILES / name))
    assert sniffed.format == MANIFEST[name]["format"]
    assert sniffed.encrypted_ooxml == (MANIFEST[name].get("encrypted", False) and name.endswith("xlsx"))


@pytest.mark.parametrize("name", EXPECTED)
def test_known_formats(name):
    assert sniff(str(FILES / name)) == EXPECTED[name]


def test_the_extension_is_ignored(copy_of):
    assert sniff(copy_of("ruled_simple.pdf", "statement.csv")).format == "pdf"
    assert sniff(copy_of("payment_history.csv", "statement.pdf")).format == "csv"
    assert sniff(copy_of("ghost_row.xls", "statement.xlsx")).format == "xls"


def test_a_plain_workbook_zip_is_xlsx_and_other_zips_are_unsupported(tmp_path):
    import openpyxl

    workbook = tmp_path / "plain.bin"
    openpyxl.Workbook().save(workbook)
    assert sniff(str(workbook)) == Sniffed("xlsx")
    other = tmp_path / "other.zip"
    import zipfile

    with zipfile.ZipFile(other, "w") as archive:
        archive.writestr("word/document.xml", "<x/>")
    assert sniff(str(other)) == Sniffed(None)


@pytest.mark.parametrize("content", [b"", b"   \n", b"\x00\x01\x02binary", b"\x89PNG\r\n\x1a\n\x00"])
def test_empty_and_binary_files_are_unsupported(tmp_path, content):
    path = tmp_path / "x.dat"
    path.write_bytes(content)
    assert sniff(str(path)) == Sniffed(None)


def test_text_with_a_byte_order_mark_is_csv(tmp_path):
    path = tmp_path / "x.dat"
    path.write_bytes("﻿a,b\n1,2\n".encode())
    assert sniff(str(path)).format == "csv"


def test_a_damaged_ole_file_is_unsupported(tmp_path):
    path = tmp_path / "x.xls"
    path.write_bytes(bytes.fromhex("d0cf11e0a1b11ae1") + b"\x00" * 64)
    assert sniff(str(path)) == Sniffed(None)
