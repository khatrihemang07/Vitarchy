from statement_reader.read.lines import group_lines, split_chunks
from statement_reader.read.pdf_words import Word


def w(text, x0, y, width=None, bold=False):
    return Word(x0, y - 4, x0 + (width or 6 * len(text)), y + 4, text, bold)


def test_words_within_tolerance_share_a_line_and_lines_run_top_to_bottom():
    words = [w("b", 50, 100), w("a", 10, 101.5), w("z", 10, 120)]
    lines = group_lines(words, page=2)
    assert [[x.text for x in line.words] for line in lines] == [["a", "b"], ["z"]]
    assert [line.page for line in lines] == [2, 2]
    assert lines[0].y < lines[1].y


def test_a_line_three_points_away_is_a_different_line():
    lines = group_lines([w("a", 10, 100), w("b", 60, 103.2)])
    assert len(lines) == 2


def test_chunks_split_where_the_gap_exceeds_the_limit_and_keep_wide_gaps_apart():
    line = group_lines([w("NET", 10, 50), w("BANKING", 32, 50), w("DEBIT", 120, 50)])[0]
    chunks = line.chunks(gap=6)
    assert [c.text for c in chunks] == ["NET BANKING", "DEBIT"]
    assert (chunks[0].x0, chunks[0].x1) == (10, 74)


def test_line_text_joins_words_with_single_spaces_and_bold_needs_every_word_bold():
    line = group_lines([w("A", 10, 50, bold=True), w("B", 30, 50, bold=True)])[0]
    assert line.text == "A B" and line.bold
    assert not group_lines([w("A", 10, 50, bold=True), w("B", 30, 50)])[0].bold


def test_split_chunks_of_nothing_is_empty():
    assert split_chunks([], 5) == []
