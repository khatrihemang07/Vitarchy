import pytest

from statement_reader.locate.stops import region_end
from statement_reader.read.lines import group_lines
from statement_reader.read.pdf_words import Word
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()


def lines_of(*texts):
    words = [Word(40, 100 + 12 * i - 4, 40 + 5 * len(t), 100 + 12 * i + 4, t) for i, t in enumerate(texts)]
    return group_lines(words)


def test_a_stop_marker_ends_the_region_on_that_line():
    lines = lines_of("01/03/2026 UPI 5.00", "# International Spends", "more")
    assert region_end(lines, 0, V) == (1, "end")


def test_a_page_total_row_ends_the_page_but_the_table_goes_on():
    lines = lines_of("01/03/2026 UPI 5.00", "Total: 5.00 0.00 95.00")
    assert region_end(lines, 0, V) == (1, "page_total")


@pytest.mark.parametrize("text", ["Statement Summary : 01-09-2026", "Your Closing Balance on 31-08-26: 1.00",
                                  "Legends Used in Account Statement", "Page no. 2", "Sincerly,"])
def test_vocabulary_markers_end_a_table(text):
    assert region_end(lines_of("a", text), 0, V) == (1, "end")


def test_without_a_marker_the_region_runs_to_the_end_of_the_page():
    lines = lines_of("a", "b")
    assert region_end(lines, 0, V) == (2, None)


def test_the_search_starts_at_the_given_line():
    lines = lines_of("Total: 1", "a", "# International Spends")
    assert region_end(lines, 1, V) == (2, "end")
