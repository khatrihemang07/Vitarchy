"""The golden harness's pure comparison: counts and enums only."""

from tests.golden.compare import compare, summarise


def tx(amount, direction):
    return {"amount_paise": amount, "direction": direction, "description": "row"}


def result(**overrides):
    base = {
        "outcome": "read", "kind": "statement", "payment_records": [],
        "statements": [{"transactions": [tx(500, "debit"), tx(200, "credit")],
                        "checks": [{"name": "balance_chain", "outcome": "passed", "detail": {}}]}],
    }
    return base | overrides


EXPECTED = {
    "outcome": "read", "kind": "statement", "statement_count": 1,
    "statements": [{"transactions": 2, "debit_paise": 500, "credit_paise": 200,
                    "checks": {"balance_chain": "passed"}}],
}


def test_a_matching_result_has_no_failures():
    assert compare(EXPECTED, result()) == []


def test_summarise_counts_and_sums_per_statement():
    summary = summarise(result())
    assert summary["statements"][0] | {"checks": None} == {
        "transactions": 2, "debit_paise": 500, "credit_paise": 200, "checks": None, "mask": None, "has_period": False}


def test_differences_are_reported_as_numbers_only():
    wrong = result(statements=[{"transactions": [tx(700, "debit")], "checks": [
        {"name": "balance_chain", "outcome": "failed", "detail": {}}]}])
    assert compare(EXPECTED, wrong) == [
        "s0.transactions: 1 != 2", "s0.debit_paise: 700 != 500", "s0.credit_paise: 0 != 200",
        "s0.check.balance_chain: failed != passed"]


def test_checks_not_listed_in_the_expectation_are_ignored():
    extra = result()
    extra["statements"][0]["checks"].append({"name": "row_counts", "outcome": "failed", "detail": {}})
    assert compare(EXPECTED, extra) == []


def test_a_stopped_file_reports_the_outcome_and_missing_statements():
    stopped = {"outcome": "no_table", "kind": None, "statements": [], "payment_records": []}
    assert compare(EXPECTED, stopped) == [
        "outcome: no_table != read", "kind: None != statement", "statement_count: 0 != 1"]


def test_payment_history_counts_and_sums():
    expected = {"outcome": "read", "kind": "payment_history", "statement_count": 0, "statements": [],
                "payment_records": {"count": 2, "debit_paise": 300, "credit_paise": 50}}
    got = {"outcome": "read", "kind": "payment_history", "statements": [],
           "payment_records": [tx(300, "debit"), tx(50, "credit")]}
    assert compare(expected, got) == []
    got["payment_records"].pop()
    assert compare(expected, got) == ["payment_records.count: 1 != 2",
                                      "payment_records.credit_paise: 0 != 50"]


def _result(*descriptions):
    rows = [{"description": d, "amount_paise": 1, "direction": "debit"} for d in descriptions]
    return {"statements": [{"transactions": rows, "checks": []}], "payment_records": []}


def test_empty_descriptions_and_descriptions_edged_by_an_amount_are_flagged_by_index_only():
    from tests.golden.compare import description_failures

    got = description_failures({}, _result("fine", "", "88565.74 UPI/1", "PART INTEREST 218,930.00", "ok 2026"))
    assert got == ["s0.row[1] empty description",
                   "s0.row[2] description starts or ends with an amount",
                   "s0.row[3] description starts or ends with an amount"]


def test_samples_must_match_exactly_and_failures_never_print_the_text():
    from tests.golden.compare import description_failures

    samples = [{"where": "s0", "row": 0, "text": "alpha"}, {"where": "s0", "row": 1, "text": "beta"}]
    got = description_failures({"description_samples": samples}, _result("alpha", "gamma"))
    assert got == ["sample[1] (s0.row[1]) differs"]


def test_a_statement_that_prints_a_period_must_yield_both_dates():
    want = {**EXPECTED, "statements": [{**EXPECTED["statements"][0], "has_period": True}]}
    dated = result()
    dated["statements"][0]["period"] = {"start": "2026-06-19", "end": "2026-07-18"}
    assert compare(want, dated) == []
    dated["statements"][0]["period"] = {"start": "2026-06-19", "end": None}
    assert compare(want, dated) == ["s0.period: start or end missing"]
    del dated["statements"][0]["period"]
    assert compare(want, dated) == ["s0.period: start or end missing"]


def test_an_expected_mask_must_match_without_printing_either_number():
    want = {**EXPECTED, "statements": [{**EXPECTED["statements"][0], "mask": "123456789012"}]}
    got = result()
    got["statements"][0]["account"] = {"mask": "123456789012"}
    assert compare(want, got) == []
    got["statements"][0]["account"] = {"mask": "999"}
    assert compare(want, got) == ["s0.mask differs"]
