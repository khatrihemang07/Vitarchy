from statement_reader.read.lines import Line
from statement_reader.read.pdf_words import Word
from statement_reader.segment.attach import attach_lines


def line(y, bold=False):
    return Line(y, [Word(0, y - 4, 10, y + 4, "x", bold)])


def owners(ys, anchors, pseudo=(), bold=(), limit=14.0, texty=None):
    lines = [line(y, i in bold) for i, y in enumerate(ys)]
    return attach_lines(lines, set(anchors), set(pseudo), limit, texty)


def test_a_line_between_two_anchors_goes_to_the_nearer_one():
    #           0    1    2    3    4
    assert owners([100, 108, 130, 136, 150], anchors=[0, 4]) == {1: 0, 2: 4, 3: 4}


def test_narration_above_the_date_line_belongs_to_the_row_below_it():
    # name line, date line (anchor), upi line, then the next row's name line and date line
    assert owners([100, 109, 118, 127, 136], anchors=[1, 4]) == {0: 1, 2: 1, 3: 4}


def test_an_equidistant_line_starting_a_bold_run_belongs_to_the_next_row():
    assert owners([100, 109, 118], anchors=[0, 2], bold={1}) == {1: 2}
    assert owners([100, 109, 118], anchors=[0, 2], texty={0, 2}) == {1: 0}


def test_an_opening_balance_row_attracts_no_narration():
    # B/F row at 100, then the first real row's name line and date line
    assert owners([100, 109, 118], anchors=[0, 2], pseudo={0}) == {1: 2}


def test_lines_far_above_the_first_anchor_or_below_the_last_are_not_part_of_any_row():
    assert owners([40, 100, 108, 150, 190], anchors=[2], limit=14) == {1: 2}


def test_a_chain_of_close_lines_below_the_last_anchor_is_kept_until_a_big_gap():
    got = owners([100, 108, 116, 150], anchors=[0], limit=14)
    assert got == {1: 0, 2: 0}


def test_with_no_anchors_nothing_is_attached():
    assert owners([100, 110], anchors=[]) == {}


def test_a_centred_row_takes_the_lines_around_its_figures_not_the_tail_of_a_one_line_row_above():
    # idx: 0 one-line row (text on its anchor), 1-2 top of the next row, 3 its figures-only anchor, 4-5 below
    got = owners([0, 10, 19, 23, 27, 36], anchors=[0, 3], texty={0})
    assert got == {1: 3, 2: 3, 4: 3, 5: 3}


def test_a_top_aligned_row_keeps_its_wrapped_tail():
    got = owners([0, 9, 18, 40, 49], anchors=[0, 3], texty={0, 3})
    assert got == {1: 0, 2: 0, 4: 3}
