from statement_reader.segment.card_sections import split_by_card
from statement_reader.segment.rows import RawRow


def row(name, card=None):
    return RawRow({}, name, 0, None, card)


def names(sections):
    return [(mask, [r.description for r in rows]) for mask, rows in ((s.mask, s.rows) for s in sections)]


def test_rows_split_into_one_section_per_card_in_printed_order():
    rows = [row("a", "1111"), row("b", "1111"), row("c", "2222")]
    assert names(split_by_card(rows, None)) == [("1111", ["a", "b"]), ("2222", ["c"])]


def test_rows_before_the_first_marker_go_to_the_bill_level_number_when_printed():
    rows = [row("a"), row("b", "2222")]
    assert names(split_by_card(rows, "1111")) == [("1111", ["a"]), ("2222", ["b"])]


def test_rows_before_the_first_marker_join_the_first_section_without_a_bill_number():
    rows = [row("a"), row("b", "2222"), row("c", "3333")]
    assert names(split_by_card(rows, None)) == [("2222", ["a", "b"]), ("3333", ["c"])]


def test_a_card_returning_after_another_stays_one_section():
    rows = [row("a", "1111"), row("b", "2222"), row("c", "1111")]
    assert names(split_by_card(rows, None)) == [("1111", ["a", "c"]), ("2222", ["b"])]


def test_without_any_marker_there_is_one_section_with_the_bill_number():
    assert names(split_by_card([row("a"), row("b")], "1111")) == [("1111", ["a", "b"])]
    assert names(split_by_card([row("a")], None)) == [(None, ["a"])]


def test_no_rows_means_no_sections():
    assert split_by_card([], "1111") == []
