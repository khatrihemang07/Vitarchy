from statement_reader.normalize.ids import is_counter, is_id_column, resolve_ids
from statement_reader.segment.rows import RawRow


def rows(field, values):
    return [RawRow({field: v}, "x", 0) for v in values]


def test_a_counter_runs_one_two_three_and_a_long_unique_column_is_an_id():
    assert is_counter(["1", "2", "3"]) and not is_counter(["1", "3"])
    assert is_id_column(["51234567907", "13687080910"]) and not is_id_column(["51234567907", "51234567907"])
    assert not is_id_column(["1", "2"])


def test_a_serial_column_of_long_unique_values_becomes_the_source_id():
    data = rows("serial", ["51234567907", "13687080910"])
    resolve_ids(data)
    assert [r.cells for r in data] == [{"source_id": "51234567907"}, {"source_id": "13687080910"}]


def test_a_source_id_column_that_only_counts_is_dropped_and_a_serial_counter_stays_a_counter():
    data = rows("source_id", ["1", "2", "3"])
    resolve_ids(data)
    assert all("source_id" not in r.cells for r in data)
    counter = rows("serial", ["1", "2"])
    resolve_ids(counter)
    assert all("source_id" not in r.cells for r in counter)
