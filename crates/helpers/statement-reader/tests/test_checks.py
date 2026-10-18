from statement_reader.checks.figures import balance_chain, chain_breaks, row_counts, summary_totals
from statement_reader.contract import Scope, Summary, Transaction


def tx_row(amount, direction, balance=None):
    return Transaction("2026-03-01", "x", amount, direction, balance)


SAVINGS = [tx_row(500, "debit", 9500), tx_row(200, "credit", 9700), tx_row(100, "debit", 9600)]


def test_a_chain_that_holds_passes_with_normal_polarity():
    check = balance_chain(SAVINGS, opening=10000)
    assert (check.name, check.outcome) == ("balance_chain", "passed")
    assert check.detail == {"rows": 3, "breaks": 0, "polarity": "normal"}


def test_the_chain_starts_at_the_first_row_when_no_opening_is_known():
    assert balance_chain(SAVINGS, opening=None).detail["rows"] == 2


def test_a_loan_chains_with_inverted_polarity():
    loan = [tx_row(1000, "credit", 21000), tx_row(300, "debit", 21300)]
    check = balance_chain(loan, opening=22000)
    assert check.outcome == "passed" and check.detail["polarity"] == "inverted"


def test_a_broken_chain_fails_and_counts_the_breaks():
    broken = [tx_row(500, "debit", 9500), tx_row(200, "credit", 9999)]
    check = balance_chain(broken, opening=10000)
    assert check.outcome == "failed" and check.detail["breaks"] == 1


def test_without_balances_the_chain_is_skipped():
    assert balance_chain([tx_row(5, "debit")], opening=None).outcome == "skipped"


def test_chain_breaks_counts_comparisons_and_breaks_for_one_polarity():
    assert chain_breaks(SAVINGS, 10000, inverted=False) == (3, 0)
    assert chain_breaks(SAVINGS, 10000, inverted=True)[1] > 0


def test_totals_must_equal_the_printed_totals_and_the_identity_must_hold():
    rows = [tx_row(500, "debit"), tx_row(200, "credit")]
    summary = Summary(opening_paise=1000, closing_paise=1300, total_debit_paise=500, total_credit_paise=200)
    assert summary_totals(rows, summary, "credit_card").outcome == "passed"
    wrong = Summary(total_debit_paise=500, total_credit_paise=300)
    check = summary_totals(rows, wrong, "credit_card")
    assert check.outcome == "failed" and check.detail == {"compared": 2, "mismatched": 1}


def test_a_savings_identity_runs_the_other_way():
    rows = [tx_row(500, "debit"), tx_row(200, "credit")]
    assert summary_totals(rows, Summary(opening_paise=1000, closing_paise=700), "savings").outcome == "passed"
    assert summary_totals(rows, Summary(opening_paise=1000, closing_paise=1300), "savings").outcome == "failed"


def test_a_closing_printed_in_whole_rupees_still_matches_within_a_rupee():
    rows = [tx_row(45340, "debit"), tx_row(1025700, "credit")]
    summary = Summary(opening_paise=1025675, closing_paise=45300)
    assert summary_totals(rows, summary, "credit_card").outcome == "passed"


def test_charges_printed_outside_the_rows_are_part_of_the_identity():
    rows = [tx_row(500, "debit")]
    summary = Summary(opening_paise=1000, closing_paise=1600)
    assert summary_totals(rows, summary, "credit_card", charges=100).outcome == "passed"


def test_totals_with_nothing_printed_are_skipped():
    assert summary_totals([tx_row(5, "debit")], Summary(), "savings").outcome == "skipped"
    assert summary_totals([tx_row(5, "debit")], Summary(opening_paise=5), "savings").outcome == "skipped"


def test_counts_compare_the_printed_debit_and_credit_counts():
    rows = [tx_row(5, "debit"), tx_row(6, "debit"), tx_row(7, "credit")]
    assert row_counts(rows, Summary(debit_count=2, credit_count=1)).outcome == "passed"
    check = row_counts(rows, Summary(debit_count=3, credit_count=1))
    assert check.outcome == "failed" and check.detail == {"compared": 2, "mismatched": 1}
    assert row_counts(rows, Summary()).outcome == "skipped"


def test_a_scope_is_reported_in_the_detail_of_totals_and_counts():
    rows = [tx_row(500, "debit")]
    summary = Summary(total_debit_paise=500, debit_count=1)
    assert summary_totals(rows, summary, "credit_card", scope=Scope.BILL).detail["scope"] == "bill"
    assert row_counts(rows, summary, scope=Scope.BILL).detail["scope"] == "bill"
    assert "scope" not in row_counts(rows, summary).detail
    assert row_counts(rows, Summary(), scope=Scope.BILL).detail["scope"] == "bill"
