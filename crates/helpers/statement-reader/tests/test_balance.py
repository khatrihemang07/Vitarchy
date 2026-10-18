from statement_reader.segment.balance import best_splits


def test_a_gap_between_a_one_line_row_and_a_centred_four_line_row_goes_to_the_lower_row():
    # Row A: text on its own anchor line (y 0). Row B: figures-only anchor at y 23, text at 10, 19, 27, 36.
    gap, after = [10.0, 19.0], [27.0, 36.0]

    def row_cost(j, above, below):
        if j == 0:
            return 0.0  # A keeps its own line wherever the gap goes (a tail below is free)
        members = (gap[len(gap) - above:] if above else []) + after
        return abs(sum(members) / len(members) - 23.0)

    assert best_splits([2], row_cost, [[0.0, 0.0, 0.0]]) == [0]


def test_no_gaps_no_splits():
    assert best_splits([], lambda *_: 0.0, []) == []


def test_distance_breaks_a_tie_and_the_upper_row_keeps_a_remaining_tie():
    assert best_splits([2], lambda *_: 0.0, [[0.5, 0.0, 0.5]]) == [1]
    assert best_splits([2], lambda *_: 0.0, [[0.0, 0.0, 0.0]]) == [2]
