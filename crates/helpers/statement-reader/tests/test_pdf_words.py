import pytest

from statement_reader.read.pdf_rules import Rule
from statement_reader.read.pdf_words import PageReader, is_bold, open_pdf

from .conftest import FILES, MANIFEST, PDFS


def read_all(name):
    reader = open_pdf(str(FILES / name))
    pages = [reader.read_page(i) for i in range(reader.page_count)]
    reader.close()
    return pages


@pytest.mark.parametrize("name", PDFS)
def test_pages_and_visible_word_counts_match_what_was_drawn(name):
    pages = read_all(name)
    assert len(pages) == MANIFEST[name]["pages"]
    assert sum(len(p.words) for p in pages) == MANIFEST[name]["words"]


def test_the_reader_satisfies_the_page_reader_seam():
    reader: PageReader = open_pdf(str(FILES / "ruled_simple.pdf"))
    assert reader.page_count == 1
    page = reader.read_page(0)
    assert (round(page.width), round(page.height)) == (595, 842)


def test_rotated_margin_text_is_dropped():
    words = [w.text for p in read_all("borderless_wrapped.pdf") for w in p.words]
    assert "42832/50000" not in words


def test_invisible_null_tokens_are_dropped():
    for name in ("headerless.pdf", "consolidated.pdf"):
        assert "null" not in [w.text for p in read_all(name) for w in p.words]


def test_a_ruled_table_yields_horizontal_and_vertical_rules():
    (page,) = read_all("ruled_simple.pdf")
    horizontal = [r for r in page.rules if r.horizontal]
    vertical = [r for r in page.rules if not r.horizontal]
    assert len(horizontal) >= 14 and len(vertical) >= 6
    assert all(isinstance(r, Rule) for r in page.rules)


def test_vector_header_blobs_are_not_rules_and_not_words():
    (page,) = read_all("headerless.pdf")
    header_band = [w for w in page.words if 120 <= w.y0 <= 144]
    assert header_band == []
    assert len([r for r in page.rules if r.horizontal]) >= 11


def test_bold_lines_are_flagged():
    (page,) = read_all("centred_narration.pdf")
    assert len([w for w in page.words if w.bold and w.text == "FAKE"]) == 7
    assert not next(w for w in page.words if w.text == "B/F").bold


def test_words_carry_geometry_in_reading_order_within_a_line():
    (page,) = read_all("ruled_simple.pdf")
    first = next(w for w in page.words if w.text == "Date")
    assert first.x0 < first.x1 and first.y0 < first.y1


@pytest.mark.parametrize("font,flags,expected", [
    ("Helvetica-Bold", 16, True), ("Mulish-Black", 0, True), ("Mulish-SemiBold", 16, False),
    ("Mulish-Regular", 0, False), ("Arial", 16, True),
])
def test_boldness_from_font_name_and_flags(font, flags, expected):
    assert is_bold(font, flags) is expected


def test_corrupt_bytes_raise_an_open_error():
    from statement_reader.errors import OpenError

    with pytest.raises(OpenError):
        open_pdf(b"%PDF-1.4 not really")
