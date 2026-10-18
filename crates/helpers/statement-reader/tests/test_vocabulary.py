import re
from pathlib import Path

import pytest

from statement_reader.vocabulary import load_vocabulary, normalise

VOCAB = load_vocabulary()
PACKAGE = Path(__file__).parent.parent / "statement_reader"


@pytest.mark.parametrize("header,field", [
    ("Date", "date"), ("Txn Date", "date"), ("Transaction Date", "date"),
    ("Value Date", "value_date"), ("Time", "time"),
    ("Transaction Details", "description"), ("PARTICULARS", "description"),
    ("Transaction Remarks", "description"), ("Details", "description"),
    ("Ref.No./Chq.No.", "reference"), ("Ref No/Cheque No", "reference"), ("UTR", "utr"),
    ("Transaction ID", "source_id"), ("Transaction Type", "type"), ("MODE", "type"),
    ("Withdrawal\nAmount (INR)", "debit"), ("Withdrawal Amount(INR)", "debit"),
    ("WITHDRAWALS", "debit"), ("Debit", "debit"), ("Credit", "credit"), ("DEPOSITS", "credit"),
    ("Deposit Amount(INR)", "credit"), ("Amount (in `)", "amount"), ("Amount", "amount"),
    ("Balance\n(INR)", "balance"), ("BALANCE", "balance"), ("Closing Balance", "balance"),
    ("Credit/debit instrument", "instrument"), ("S No.", "serial"), ("SerNo.", "source_id"),
    ("Intl.# amount", "foreign_amount"), ("Reward Points", "ignored"),
])
def test_verbatim_headers_from_real_statements_map_to_their_field(header, field):
    assert VOCAB.field_of(header) == field


@pytest.mark.parametrize("cell", ["", "Dr Count", "Total Debits", "Statement Summary", "Foo"])
def test_cells_that_are_not_headers_map_to_nothing(cell):
    assert VOCAB.field_of(cell) is None


def test_a_cell_naming_both_credit_and_debit_is_never_a_credit_or_debit_column():
    assert VOCAB.field_of("Credit/debit instrument extra") is None


def test_no_synonym_belongs_to_two_fields():
    seen: dict[str, str] = {}
    for field, words in VOCAB.fields.items():
        for word in words:
            assert seen.setdefault(normalise(word), field) == field, word


def test_dates_and_directions_carry_the_formats_and_words_the_reports_found():
    assert VOCAB.month_aliases["sept"] == "sep"
    assert "%d-%m-%y" in VOCAB.date_formats and "%b %d, %Y" in VOCAB.date_formats
    assert {"cr", "c"} <= set(VOCAB.directions["credit_suffixes"])
    assert {"debit", "credit"} <= set(VOCAB.directions["debit_words"] + VOCAB.directions["credit_words"])


@pytest.mark.parametrize("line", [
    "Total: 40,000.00 25,601.00", "Statement Summary : 01-09-2026", "Legends Used in Account Statement",
    "# International Spends", "Page no. 2", "Your Closing Balance on 31-08-26: 1.00",
])
def test_stop_markers_recognise_the_lines_that_end_a_table(line):
    assert VOCAB.is_stop_line(line)


def test_a_transaction_line_is_not_a_stop_line():
    assert not VOCAB.is_stop_line("01-08-2026 UPI/512345678986/DEMO GRO 3,500.00")


@pytest.mark.parametrize("description,reference", [
    ("UPI/DR/512345678986/DEMO GRO/HDFC/xyautopay./India", "512345678986"),
    ("UPI-512345679003-PINE TRE E FUELS IN", "512345679003"),
    ("UPI/512345679037/NA/someone@p/State Bank Of I/PTM", "512345679037"),
    ("NEFT*KKBK0000958*CMS5123456789131*Fakeship Limit", "CMS5123456789131"),
    ("IMPS/412345678901/x", "412345678901"),
])
def test_reference_patterns_find_the_reference_in_group_one(description, reference):
    found = [m.group(1) for p in VOCAB.reference_patterns if (m := p.search(description))]
    assert found[0] == reference


def test_section_patterns_find_masked_numbers_and_account_labels():
    assert VOCAB.masked_number.search("4123XXXXXXXX2003").group(0) == "4123XXXXXXXX2003"
    assert VOCAB.masked_account.search("DL/TL ACCOUNT xxxxxxx1470")
    assert VOCAB.cardholder_divider.search("TRANSACTIONS FOR SOME NAME")
    kinds = {kind for pattern, kind in VOCAB.account_label_kinds if pattern.search("DL/TL ACCOUNT")}
    assert kinds == {"loan"}


def test_issuer_markers_name_the_issuer_and_default_kind():
    by_name = {i.name: i for i in VOCAB.issuers}
    assert by_name["Example Bank"].ifsc_prefix == ("EXMP",) and by_name["Example Bank"].kind == "savings"
    assert all(i.kind in {"savings", "credit_card", "loan"} for i in VOCAB.issuers)


def test_python_code_names_no_bank_or_issuer():
    banned = [n for i in VOCAB.issuers for n in i.names] + ["sbi", "icici", "phonepe", "bobcard"]
    pattern = re.compile("|".join(re.escape(n) for n in banned), re.IGNORECASE)
    offenders = [p.name for p in PACKAGE.rglob("*.py") if pattern.search(p.read_text())]
    assert offenders == []
