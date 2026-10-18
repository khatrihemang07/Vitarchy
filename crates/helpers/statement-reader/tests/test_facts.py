from datetime import date

from statement_reader.metadata.facts import read_facts
from statement_reader.metadata.issuer import detect_issuer, detect_kind
from statement_reader.metadata.surface import FactCell, Surface
from statement_reader.vocabulary import load_vocabulary

V = load_vocabulary()


def surface(*cells):
    return Surface([FactCell(*c) for c in cells], 40.0, 2.5)


def test_issuer_comes_from_the_earliest_name_ifsc_prefix_or_gstin():
    assert detect_issuer("welcome to Example Bank, State Bank of India partner", V).name == "Example Bank"
    assert detect_issuer("IFSC Code : EXMP0001234", V).name == "Example Bank"
    assert detect_issuer("nothing here", V) is None


def test_kind_is_the_earliest_vocabulary_phrase_else_the_issuers_usual_kind():
    issuer = detect_issuer("Example Bank", V)
    assert detect_kind("CREDIT CARD STATEMENT then savings account", V, issuer) == "credit_card"
    assert detect_kind("Statement of Transactions in Savings Account", V, issuer) == "savings"
    assert detect_kind("DL/TL ACCOUNT xxxxxxx1470 loan account", V, None) == "loan"
    assert detect_kind("unrelated", V, issuer) == "savings" and detect_kind("unrelated", V, None) is None


def test_a_card_summary_box_is_read_by_labels_with_values_beneath_them():
    s = surface(
        ("Example Bank", 40, 120, 60.0), ("CREDIT CARD STATEMENT", 40, 160, 76.0),
        ("Card Number 4000XXXXXXXX1111", 40, 200, 90.0),
        ("Statement period : June 29, 2026 to July 28, 2026", 40, 300, 102.0),
        ("Previous Balance", 40, 110, 140.0), ("Purchases", 160, 200, 140.0), ("Payments", 280, 330, 140.0),
        ("Total Amount Due", 400, 480, 140.0),
        ("50,000.00", 40, 90, 156.0), ("36,496.60", 160, 210, 156.0), ("4,596.33", 280, 330, 156.0),
        ("81,900.27", 400, 450, 156.0))
    facts = read_facts(s, V)
    assert facts.summary.opening_paise == 5000000 and facts.summary.closing_paise == 8190027
    assert (facts.summary.total_debit_paise, facts.summary.total_credit_paise) == (3649660, 459633)
    assert facts.period == (date(2026, 6, 29), date(2026, 7, 28))
    assert (facts.mask, facts.issuer, facts.kind) == ("4000XXXXXXXX1111", "Example Bank", "credit_card")


def test_heading_values_override_the_document_search_for_mask_and_kind():
    s = surface(("Account Number : XXXXXX0001", 40, 200, 60.0), ("Statement of Account", 40, 200, 80.0))
    facts = read_facts(s, V, heading_mask="xxxxxxx2222", heading_kind="loan")
    assert (facts.mask, facts.kind) == ("xxxxxxx2222", "loan")


def test_a_balance_printed_for_today_is_not_the_closing_balance():
    s = surface(("Clear Balance : 59,445.01CR", 40, 200, 60.0), ("Account Balance", 40, 120, 80.0),
                ("46,379.62", 40, 90, 94.0))
    facts = read_facts(s, V)
    assert facts.summary.closing_paise is None and facts.summary.opening_paise is None


def test_nothing_printed_gives_empty_facts():
    facts = read_facts(surface(("hello", 0, 10, 10.0)), V)
    assert facts.summary.opening_paise is None and facts.period == (None, None)
    assert facts.mask is None and facts.charges == 0


def test_a_number_printed_in_the_header_beats_the_section_marker_lines_before_it():
    s = surface(("3123XXXXXXXX2580", 40, 140, 50.0), ("Your card 4123XXXXXXXX2003 summary", 40, 240, 80.0),
                ("5123XXXXXXXX1357", 40, 140, 120.0))
    assert read_facts(s, V).mask == "4123XXXXXXXX2003"


def test_without_a_statement_level_number_the_first_section_marker_is_the_mask():
    s = surface(("3123XXXXXXXX2580", 40, 140, 50.0), ("5123XXXXXXXX1357", 40, 140, 120.0))
    assert read_facts(s, V).mask == "3123XXXXXXXX2580"


def test_a_period_printed_on_its_own_line_with_no_label_is_read():
    s = surface(("19 May, 2026 - 18 Jun, 2026", 40, 200, 40.0), ("29/05/2026 UPI-X DEBIT 127", 40, 300, 60.0))
    assert read_facts(s, V).period == (date(2026, 5, 19), date(2026, 6, 18))


def test_a_number_after_a_label_inside_a_sentence_is_the_mask():
    s = surface(("Statement of Transactions in Saving Account no. 787001508765 in INR for the period "
                 "January 1, 2023 - December 31, 2023", 40, 500, 60.0))
    assert read_facts(s, V).mask == "787001508765"


def test_a_label_in_prose_never_picks_up_a_date_an_amount_or_a_short_run():
    for text in ("Your account number is 01012026 on file", "account no. 1,00,000.00 due",
                 "account number 1234567890123456789 ref",
                 "send SMS with card number> to 9000000000 now"):
        assert read_facts(surface((text, 40, 300, 60.0)), V).mask is None
